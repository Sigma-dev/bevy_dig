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
};

use crate::TerrainVertices;
const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

pub const CHUNK_WIDTH: usize = 32;
const BUFFER_LEN: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;
const MAX_VERTICES_PER_CUBE: usize = 12;
const TRI_BUFFER_LEN: usize = BUFFER_LEN * MAX_VERTICES_PER_CUBE;

pub(crate) struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ReadbackBuffer>::default())
            .add_systems(Startup, setup);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputePipeline>().add_systems(
            Render,
            prepare_bind_group
                .in_set(RenderSet::PrepareBindGroups)
                .run_if(not(resource_exists::<GpuBufferBindGroup>)),
        );

        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

pub fn index_to_coordinates(index: usize) -> Vec3 {
    return Vec3::new(
        index as f32 % CHUNK_WIDTH as f32,
        (index as f32 / CHUNK_WIDTH as f32) % CHUNK_WIDTH as f32,
        index as f32 / (CHUNK_WIDTH as f32 * CHUNK_WIDTH as f32),
    );
}

pub fn make_sphere_buffer() -> Vec<u32> {
    let radius = CHUNK_WIDTH as f32 / 2.0;
    let mut vec = vec![0u32; BUFFER_LEN];
    for (i, e) in vec.iter_mut().enumerate() {
        let pos = index_to_coordinates(i);
        let center = Vec3::new(radius, radius, radius);
        let dist = pos.distance(center);
        if dist < radius as f32 / 2. {
            *e = 1;
        }
    }
    vec
}

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer(Handle<ShaderStorageBuffer>, Handle<ShaderStorageBuffer>);

fn setup(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    let mut input_buffer = ShaderStorageBuffer::from(make_sphere_buffer());
    input_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let mut output_buffer = ShaderStorageBuffer::from(vec![Vec4::ZERO; TRI_BUFFER_LEN]);
    output_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let input_handle = buffers.add(input_buffer);
    let output_handle = buffers.add(output_buffer);

    commands
        .spawn(Readback::buffer(output_handle.clone()))
        .observe(
            |trigger: Trigger<ReadbackComplete>, mut terrain_vertices: ResMut<TerrainVertices>| {
                let data: Vec<Vec4> = trigger.event().to_shader_type();
                let filtered: Vec<&Vec4> = data.iter().filter(|v| **v != Vec4::splat(0.)).collect();

                let vertices: Vec<Vec3> = filtered.iter().map(|v4| v4.xyz()).collect();
                if terrain_vertices.0.len() == 0 {
                    *terrain_vertices = TerrainVertices(vertices);
                }
            },
        );
    commands.insert_resource(ReadbackBuffer(input_handle, output_handle));
}

#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffer: Res<ReadbackBuffer>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let input_buffer: &GpuShaderStorageBuffer = buffers.get(&buffer.0).unwrap();
    let output_buffer: &GpuShaderStorageBuffer = buffers.get(&buffer.1).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            input_buffer.buffer.as_entire_buffer_binding(),
            output_buffer.buffer.as_entire_buffer_binding(),
        )),
    );
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
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(BUFFER_LEN as u32, 1, 1);
        }
        Ok(())
    }
}
