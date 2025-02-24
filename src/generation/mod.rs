use bevy::{
    prelude::*,
    render::{
        extract_resource::*,
        gpu_readback::*,
        mesh::Indices,
        render_asset::*,
        render_graph::*,
        render_resource::{binding_types::storage_buffer, *},
        renderer::*,
        storage::*,
        *,
    },
    utils::hashbrown::HashMap,
};

const SHADER_ASSET_PATH: &str = "shaders/marching_cubes.wgsl";

pub const CHUNK_WIDTH: usize = 32;
pub const CHUNK_DATA: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;
pub const INPUT_CHUNK_WIDTH: usize = CHUNK_WIDTH + 2;
pub const BUFFER_LEN: usize = INPUT_CHUNK_WIDTH * INPUT_CHUNK_WIDTH * INPUT_CHUNK_WIDTH;
const MAX_VERTICES_PER_CUBE: usize = 12;
const TRI_BUFFER_LEN: usize =
    (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * MAX_VERTICES_PER_CUBE;

pub(crate) struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExtractResourcePlugin::<ReadbackBuffer>::default(),))
            .add_event::<ChunkMeshGenerated>()
            .add_systems(Startup, setup)
            .add_systems(PostUpdate, update_resource);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputePipeline>().add_systems(
            Render,
            prepare_bind_group
                .in_set(RenderSet::PrepareBindGroups)
                .run_if(resource_exists::<BuildTerrain>),
        );

        render_app
            .add_systems(ExtractSchedule, extract_build_terrain)
            .insert_resource::<FirstBuild>(FirstBuild)
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

#[derive(Event)]
pub struct ChunkMeshGenerated {
    pub index: UVec3,
    pub mesh: Mesh,
}

impl ChunkMeshGenerated {
    pub fn new(index: UVec3, mesh: Mesh) -> ChunkMeshGenerated {
        ChunkMeshGenerated { index, mesh }
    }
}

#[derive(Component)]
pub struct ReadBackIndex(UVec3);

#[derive(Resource, Debug)]
pub struct TerrainData(pub Handle<ShaderStorageBuffer>, pub UVec3);

fn update_resource(
    terrain_data: Option<Res<TerrainData>>,
    mut commands: Commands,
    maybe_buffer: Option<ResMut<ReadbackBuffer>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.remove_resource::<BuildTerrain>();
    let Some(res) = terrain_data else {
        return;
    };

    if res.is_changed() {
        let Some(mut buffer) = maybe_buffer else {
            return;
        };
        commands.insert_resource::<BuildTerrain>(BuildTerrain);
        buffer.input = res.0.clone();
        buffers.insert(&buffer.output, make_empty_triangles_buffer());

        commands
            .spawn((
                Readback::buffer(buffer.output.clone()),
                ReadBackIndex(res.1),
            ))
            .observe(
                |trigger: Trigger<ReadbackComplete>,
                 mut commanads: Commands,
                 mut chunk_mesh_w: EventWriter<ChunkMeshGenerated>,
                 index_q: Query<&ReadBackIndex>| {
                    let index = index_q.get(trigger.entity()).unwrap().0;
                    let readback: Vec<Vec4> = trigger.event().to_shader_type();
                    let filtered: Vec<Vec3> = readback
                        .iter()
                        .filter(|v| v.w != -1.)
                        .map(|v4| v4.xyz())
                        .collect();
                    let (indices, unique) = deduplicate_vertices(&filtered, 0.1);
                    println!("Readback {:?}", indices.len());
                    if indices.len() > 0 {
                        let mesh = create_terrain_mesh(&indices, &unique);
                        chunk_mesh_w.send(ChunkMeshGenerated::new(index, mesh));
                        commanads.entity(trigger.entity()).despawn();
                    }
                },
            );
    }
}

pub fn create_terrain_mesh(indices: &Vec<usize>, uniques: &Vec<Vec3>) -> Mesh {
    let indices_u32: Vec<u32> = indices.iter().map(|i| *i as u32).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, uniques.clone())
    .with_inserted_indices(Indices::U32(indices_u32))
    .with_computed_normals()
}

trait SpatialHash {
    fn spatial_hash(&self, epsilon: f32) -> (i32, i32, i32);
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool;
}

impl SpatialHash for Vec3 {
    fn spatial_hash(&self, epsilon: f32) -> (i32, i32, i32) {
        (
            (self.x / epsilon).round() as i32,
            (self.y / epsilon).round() as i32,
            (self.z / epsilon).round() as i32,
        )
    }
    fn approx_eq(&self, other: &Vec3, epsilon: f32) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
    }
}

fn deduplicate_vertices(vec: &Vec<Vec3>, epsilon: f32) -> (Vec<usize>, Vec<Vec3>) {
    let mut unique_pos: Vec<Vec3> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();
    let mut hash_map: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();

    for pos in vec {
        let hash = pos.spatial_hash(epsilon);
        let mut found_index = None;

        if let Some(bucket) = hash_map.get(&hash) {
            for &index in bucket {
                if unique_pos[index].approx_eq(pos, epsilon) {
                    found_index = Some(index);
                    break;
                }
            }
        }

        if let Some(index) = found_index {
            indices.push(index);
        } else {
            let new_index = unique_pos.len();
            unique_pos.push(*pos);
            indices.push(new_index);
            hash_map
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(new_index);
        }
    }

    (indices, unique_pos)
}

#[derive(Resource, ExtractResource, Clone, Debug)]
pub struct ReadbackBuffer {
    input: Handle<ShaderStorageBuffer>,
    output: Handle<ShaderStorageBuffer>,
}

impl ReadbackBuffer {
    pub fn new(
        input: Handle<ShaderStorageBuffer>,
        output: Handle<ShaderStorageBuffer>,
    ) -> ReadbackBuffer {
        ReadbackBuffer { input, output }
    }
}

fn make_empty_triangles_buffer() -> ShaderStorageBuffer {
    let mut output_buffer =
        ShaderStorageBuffer::from(vec![Vec4::new(0., 0., 0., -1.); TRI_BUFFER_LEN]);
    output_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    output_buffer
}

fn setup(mut commands: Commands, buffers: Res<Assets<ShaderStorageBuffer>>) {
    commands.insert_resource(ReadbackBuffer::new(
        buffers.reserve_handle(),
        buffers.reserve_handle(),
    ));
}

#[derive(Resource, Debug, Default)]
struct BuildTerrain;

fn extract_build_terrain(
    mut commands: Commands,
    build_terrain: Extract<Option<Res<BuildTerrain>>>,
) {
    if build_terrain.is_some() {
        commands.init_resource::<BuildTerrain>();
    } else {
        commands.remove_resource::<BuildTerrain>();
    }
}

#[derive(Resource, Debug)]
struct FirstBuild;

#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffer: Res<ReadbackBuffer>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let input_buffer: &GpuShaderStorageBuffer = buffers.get(&buffer.input).unwrap();
    let output_buffer: &GpuShaderStorageBuffer = buffers.get(&buffer.output).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            input_buffer.buffer.as_entire_buffer_binding(),
            output_buffer.buffer.as_entire_buffer_binding(),
        )),
    );
    println!("Bind Prepared");
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<[u32; BUFFER_LEN]>(false),
                    storage_buffer::<[Vec4; TRI_BUFFER_LEN]>(false),
                ),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("GPU readback compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        ComputePipeline { layout, pipeline }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

#[derive(Default)]
struct ComputeNode {}
impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if world.get_resource::<BuildTerrain>().is_none() {
            return Ok(());
        }

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            println!("Passed");
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(
                (CHUNK_WIDTH + 1) as u32,
                (CHUNK_WIDTH + 1) as u32,
                (CHUNK_WIDTH + 1) as u32,
            );
        }
        Ok(())
    }
}
