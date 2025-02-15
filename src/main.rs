use bevy::{
    asset::RenderAssetUsages,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::mouse,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use generation::{make_sphere_buffer, GpuReadbackPlugin, ReadBackMarker};

mod generation;

fn main() {
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
        .insert_resource(TerrainData(Vec::new()))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_mesh, handle_inputs))
        .run();
}

#[derive(Resource, Debug)]
pub struct TerrainVertices(Vec<Vec3>);

#[derive(Resource, Debug)]
pub struct TerrainData(Vec<bool>);

fn handle_inputs(
    mut terrain: ResMut<TerrainData>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    readback_q: Query<&ReadBackMarker>,
    time: Res<Time>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let mult = (time.elapsed_secs().sin() + 1.) / 2.;
        terrain.0 = make_sphere_buffer(mult);
    }
}

fn setup(mut commands: Commands, terrain_vertices: Res<TerrainVertices>) {
    commands.spawn((Camera3d::default(), EditorCam::default()));

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
}

fn update_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_vertices: Res<TerrainVertices>,
) {
    if !terrain_vertices.is_changed() || terrain_vertices.0.len() == 0 {
        return;
    }
    let terrain_mesh_handle: Handle<Mesh> = meshes.add(create_terrain_mesh(&terrain_vertices.0));

    println!("building mesh");
    commands.spawn((
        Mesh3d(terrain_mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
    ));
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
