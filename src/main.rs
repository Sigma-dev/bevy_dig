use bevy::{core::FrameCount, prelude::*};
use bevy_editor_cam::DefaultEditorCamPlugins;
use generation::*;
use std::collections::VecDeque;
mod generation;

pub const VOXEL_SCALE: f32 = 0.25;

#[derive(Resource)]
pub struct ChunksToGenerateQueue(pub VecDeque<ChunksToGenerateQueueElement>);

pub struct ChunksToGenerateQueueElement {
    pub index: UVec3,
    pub input_data: [bool; BUFFER_LEN_UNCOMPRESSED],
}

#[derive(Component)]
struct ChunkMesh {
    index: UVec3,
}

fn main() {
    // simulate_shader::run_simulation();
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins, GpuReadbackPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (delayed_setup, update_mesh))
        .insert_resource(ChunksToGenerateQueue(VecDeque::new()))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::splat(15.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn delayed_setup(mut queue: ResMut<ChunksToGenerateQueue>, frame_count: Res<FrameCount>) {
    if frame_count.0 != 20 {
        return;
    }
    let mut chunk = [true; BUFFER_LEN_UNCOMPRESSED];
    for i in 0..BUFFER_LEN_UNCOMPRESSED {
        chunk[i] = i % 2 == 0;
    }
    queue.0.push_back(ChunksToGenerateQueueElement {
        index: UVec3::ZERO,
        input_data: chunk,
    });
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_chunk_r: EventReader<ChunkMeshGenerated>,
    terrain_q: Query<(&Mesh3d, &ChunkMesh)>,
) {
    for ev in mesh_chunk_r.read() {
        let scale = VOXEL_SCALE;
        let mesh = ev.mesh.clone().scaled_by(Vec3::splat(scale));

        let _material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.2, 0.2),
            perceptual_roughness: 1.,
            ..default()
        });

        if let Some((mesh_handle, _)) = terrain_q.iter().find(|(_, chunk)| chunk.index == ev.index)
        {
            meshes.insert(mesh_handle, mesh);
        } else {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
                ChunkMesh { index: ev.index },
            ));
        }
    }
}
