use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{storage_buffer, storage_buffer_read_only},
            *,
        },
        renderer::*,
        *,
    },
};

const SHADER_ASSET_PATH: &str = "shaders/marching_cubes.wgsl";
pub const CHUNK_WIDTH: usize = 31;
pub const INPUT_CHUNK_WIDTH: usize = CHUNK_WIDTH + 2;
pub const BUFFER_LEN_UNCOMPRESSED: usize =
    INPUT_CHUNK_WIDTH * INPUT_CHUNK_WIDTH * INPUT_CHUNK_WIDTH;
pub const BUFFER_LEN: usize = BUFFER_LEN_UNCOMPRESSED / 32;
const MAX_VERTICES_PER_CUBE: usize = 12;
const TRI_BUFFER_LEN: usize =
    (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2) * MAX_VERTICES_PER_CUBE;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .run();
}

pub(crate) struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let world = render_app.world();
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<[u32; BUFFER_LEN]>(false),
                    storage_buffer::<[Vec4; TRI_BUFFER_LEN]>(false),
                    storage_buffer_read_only::<[[i32; 16]; 256]>(false),
                ),
            ),
        );
        let shader: Handle<Shader> = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let _pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("GPU readback compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
    }
}
