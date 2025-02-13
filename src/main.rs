use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use generation::GpuReadbackPlugin;

mod generation;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin,
            MeshPickingPlugin,
            DefaultEditorCamPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_mesh)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TerrainVertices(Vec::new()))
        .run();
}

#[derive(Resource)]
pub struct TerrainVertices(Vec<Vec3>);

fn setup(mut commands: Commands) {
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
