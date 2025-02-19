use std::{collections::BTreeMap, ops::Index};

use bevy::{
    prelude::*,
    render::{
        extract_resource::*,
        gpu_readback::*,
        render_asset::*,
        render_graph::*,
        render_resource::{binding_types::storage_buffer, *},
        renderer::*,
        storage::*,
        *,
    },
    utils::hashbrown::{HashMap, HashSet},
};

use crate::{voxel::VoxelChunk, TerrainData, TerrainVertices};
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

pub const CHUNK_WIDTH: usize = 32;
pub const BUFFER_LEN: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;
const MAX_VERTICES_PER_CUBE: usize = 12;
const TRI_BUFFER_LEN: usize =
    (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * MAX_VERTICES_PER_CUBE;

pub(crate) struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ReadbackBuffer>::default())
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

#[derive(Component)]
pub struct ReadBackMarker;

fn update_resource(
    terrain_data: Option<Res<TerrainData>>,
    mut commands: Commands,
    maybe_buffer: Option<ResMut<ReadbackBuffer>>,
    maybe_first: Option<Res<FirstBuild>>,
    maybe_trigger: Option<Res<BuildTerrain>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.remove_resource::<BuildTerrain>();
    let Some(res) = terrain_data else {
        return;
    };
    let is_changed = res.is_changed();
    let data = res.0.clone();

    if is_changed || maybe_first.is_some() {
        let Some(mut buffer) = maybe_buffer else {
            return;
        };
        commands.remove_resource::<FirstBuild>();
        println!("changed: {:?}", is_changed);
        commands.insert_resource::<BuildTerrain>(BuildTerrain);
        buffer.input = data.clone();
        buffers.insert(&buffer.output, make_empty_triangles_buffer());

        let id = commands
            .spawn((Readback::buffer(buffer.output.clone()), ReadBackMarker))
            .observe(
                |trigger: Trigger<ReadbackComplete>,
                 mut terrain_vertices: ResMut<TerrainVertices>,
                 mut commanads: Commands| {
                    let data: Vec<Vec4> = trigger.event().to_shader_type();
                    let filtered: Vec<&Vec4> =
                        data.iter().filter(|v| **v != Vec4::splat(0.)).collect();
                    let (indices, unique) = deduplicate_vertices(&filtered, 0.1);

                    let vertices: Vec<Vec3> = filtered.iter().map(|v4| v4.xyz()).collect();
                    let uniques: Vec<Vec3> = unique.iter().map(|v4| v4.xyz()).collect();
                    println!("Readback {:?}", vertices.len());
                    if vertices.len() > 0 {
                        *terrain_vertices = TerrainVertices(vertices, indices, uniques);
                        println!("despawn");
                        commanads.entity(trigger.entity()).despawn();
                    }
                },
            )
            .id();
    }
}

/// Trait for tolerance-based equality comparison.
trait ApproxEq {
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool;
}

/// Trait for spatial hashing.
trait SpatialHash {
    fn spatial_hash(&self, epsilon: f32) -> (i32, i32, i32, i32);
}

impl ApproxEq for Vec4 {
    /// Compare two `Vec4` values with a tolerance for floating-point inaccuracies.
    fn approx_eq(&self, other: &Vec4, epsilon: f32) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
            && (self.w - other.w).abs() < epsilon
    }
}

impl SpatialHash for Vec4 {
    /// Spatial hash function: rounds coordinates to a grid of size `epsilon`.
    fn spatial_hash(&self, epsilon: f32) -> (i32, i32, i32, i32) {
        (
            (self.x / epsilon).round() as i32,
            (self.y / epsilon).round() as i32,
            (self.z / epsilon).round() as i32,
            (self.w / epsilon).round() as i32,
        )
    }
}

/* fn deduplicate_vertices(vec: &Vec<&Vec4>) -> (Vec<usize>, Vec<Vec4>) {
    let mut unique_pos: Vec<Vec4> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    for pos in vec {
        if let Some(index) = unique_pos.iter().position(|p| p == *pos) {
            indices.push(index)
        } else {
            unique_pos.push(**pos);
            indices.push(unique_pos.len() - 1);
        }
    }
    /*
    let test: Vec<Vec4> = indices.iter().map(|i| unique_floats[*i]).collect();

    let test2: Vec<Vec4> = vec.iter().map(|v| **v).collect();
    if (test2 == test) {
        println!("yes !");
    } */
    return (indices, unique_pos);
} */
fn deduplicate_vertices(vec: &Vec<&Vec4>, epsilon: f32) -> (Vec<usize>, Vec<Vec4>) {
    let mut unique_pos: Vec<Vec4> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();
    let mut hash_map: HashMap<(i32, i32, i32, i32), Vec<usize>> = HashMap::new();

    for pos in vec {
        let hash = pos.spatial_hash(epsilon);
        let mut found_index = None;

        // Check only vertices in the same spatial hash bucket
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
            unique_pos.push(**pos);
            indices.push(new_index);
            hash_map
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(new_index);
        }
    }

    (indices, unique_pos)
}

pub fn index_to_coordinates(index: usize) -> Vec3 {
    return Vec3::new(
        index as f32 % CHUNK_WIDTH as f32,
        (index as f32 / CHUNK_WIDTH as f32) % CHUNK_WIDTH as f32,
        index as f32 / (CHUNK_WIDTH as f32 * CHUNK_WIDTH as f32),
    );
}

pub fn make_sphere_buffer(radius_mult: f32) -> Vec<bool> {
    let radius = CHUNK_WIDTH as f32 / 2.0 * radius_mult;
    let mut vec = vec![false; BUFFER_LEN];
    for (i, e) in vec.iter_mut().enumerate() {
        let pos = index_to_coordinates(i);
        let center = Vec3::new(radius, radius, radius);
        let dist = pos.distance(center);
        if dist < radius as f32 / 2. {
            *e = true;
        }
    }
    vec
}

pub fn make_full_buffer() -> Vec<bool> {
    let mut vec = vec![false; BUFFER_LEN];
    for (i, e) in vec.iter_mut().enumerate() {
        if i < BUFFER_LEN {
            *e = true;
        }
    }
    vec
}

pub fn convert_booleans_to_buffer(booleans: &Vec<bool>) -> Vec<u32> {
    booleans.iter().map(|a| if *a { 1 } else { 0 }).collect()
}

pub fn convert_booleans_arr_to_buffer(booleans: &[bool; BUFFER_LEN as usize]) -> Vec<u32> {
    booleans.iter().map(|a| if *a { 1 } else { 0 }).collect()
}

#[derive(Resource, ExtractResource, Clone, Debug)]
pub struct ReadbackBuffer {
    input: Handle<ShaderStorageBuffer>,
    output: Handle<ShaderStorageBuffer>,
    ran_first: bool,
}

impl ReadbackBuffer {
    pub fn new(
        input: Handle<ShaderStorageBuffer>,
        output: Handle<ShaderStorageBuffer>,
    ) -> ReadbackBuffer {
        ReadbackBuffer {
            input,
            output,
            ran_first: false,
        }
    }
}

fn make_empty_triangles_buffer() -> ShaderStorageBuffer {
    let mut output_buffer = ShaderStorageBuffer::from(vec![Vec4::ZERO; TRI_BUFFER_LEN]);
    output_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    output_buffer
}

fn setup(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    let mut input_buffer =
        ShaderStorageBuffer::from(convert_booleans_to_buffer(&make_sphere_buffer(1.)));
    /*  let mut input_buffer =
    ShaderStorageBuffer::from(convert_booleans_to_buffer(&VoxelChunk::empty().raw())); */
    input_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let output_buffer = make_empty_triangles_buffer();
    let input_handle = buffers.add(input_buffer);
    let output_handle = buffers.add(output_buffer);
    println!("output: {:?}", output_handle);

    commands.insert_resource(ReadbackBuffer::new(input_handle, output_handle));
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
    println!("{:?}", output_buffer.buffer);
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
                    storage_buffer::<Vec<u32>>(false),
                    storage_buffer::<Vec<Vec4>>(false),
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
        let buffer = world.resource::<ReadbackBuffer>();

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
                CHUNK_WIDTH as u32 + 2,
                CHUNK_WIDTH as u32 + 2,
                CHUNK_WIDTH as u32 + 2,
            );
        }
        Ok(())
    }
}
