use bevy::{
    asset::RenderAssetUsages,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    ecs::bundle,
    input::mouse,
    math::VectorSpace,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::BufferUsages,
        storage::ShaderStorageBuffer,
    },
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use generation::{
    convert_booleans_to_buffer, make_full_buffer, make_sphere_buffer, GpuReadbackPlugin,
    ReadBackMarker, CHUNK_WIDTH,
};
use simulate_shader::run_simulation;

mod generation;
mod simulate_shader;

fn main() {
    //let data = run_simulation();

    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                    },
                    enabled: true,
                    ..default()
                },
            },
            GpuReadbackPlugin,
            MeshPickingPlugin,
            DefaultEditorCamPlugins,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TerrainVertices(Vec::new()))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_mesh, handle_inputs))
        //.add_systems(Update, draw_simulation)
        .run();
}

#[derive(Resource, Debug)]
pub struct TerrainVertices(Vec<Vec3>);

#[derive(Resource, Debug)]
pub struct TerrainData(Handle<ShaderStorageBuffer>);

fn handle_inputs(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    readback_q: Query<&ReadBackMarker>,
    time: Res<Time>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let mult = (time.elapsed_secs().sin() + 1.) / 2.;
        let mut input_buffer =
            ShaderStorageBuffer::from(convert_booleans_to_buffer(&make_full_buffer()));
        /*     let mut input_buffer =
        ShaderStorageBuffer::from(convert_booleans_to_buffer(&&make_sphere_buffer(mult))); */
        input_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let handle = buffers.add(input_buffer);
        commands.insert_resource(TerrainData(handle));
    }
}

fn setup(
    mut commands: Commands,
    terrain_vertices: Res<TerrainVertices>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(),
        Transform::from_translation(Vec3::splat(50.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_x(-30_f32.to_radians()),
            ..default()
        },
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE.into(),
        brightness: 100.,
    });

    commands.spawn((
        Mesh3d(meshes.reserve_handle()),
        MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
        Terrain,
        Transform::from_translation(-Vec3::splat(CHUNK_WIDTH as f32) / 2.),
    ));
}

#[derive(Component)]
struct Terrain;

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_vertices: Res<TerrainVertices>,
    mut terrain_q: Query<&mut Mesh3d, With<Terrain>>,
) {
    if !terrain_vertices.is_changed() || terrain_vertices.0.len() == 0 {
        return;
    }
    for terrain in terrain_q.iter() {
        meshes.insert(&terrain.0, create_terrain_mesh(&terrain_vertices.0));
        //let terrain_mesh_handle: Handle<Mesh> = meshes.add(create_terrain_mesh(&terrain_vertices.0));
    }

    println!("building mesh");
}

fn create_terrain_mesh(terrain_vertices: &Vec<Vec3>) -> Mesh {
    let triangles: Vec<[f32; 3]> = terrain_vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let indices = terrain_vertices
        .iter()
        .enumerate()
        .map(|(i, _)| i as u32)
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, triangles)
    .with_inserted_indices(Indices::U32(indices));
    mesh.compute_smooth_normals();
    mesh
}

fn draw_simulation(mut gizmos: Gizmos) {
    let data = run_simulation();
    for i in 0..(data.len() - 1) {
        let pos = data[i];
        if pos != Vec4::ZERO {
            gizmos.sphere(Isometry3d::from_translation(pos.xyz()), 1., Color::WHITE);
        }
    }
    //println!("{:?}", data);
}
