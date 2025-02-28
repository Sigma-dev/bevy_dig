use std::collections::VecDeque;

use avian3d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, window::PrimaryWindow};
use interaction::{PointerPosition, VoxelInteractionPlugin};

use crate::{
    generation::{ChunkMeshGenerated, GpuReadbackPlugin, BUFFER_LEN_UNCOMPRESSED, CHUNK_WIDTH},
    voxel::{chunks_manager::ChunksManager, VoxelChunk},
};

mod interaction;

pub const VOXEL_SCALE: f32 = 0.25;

#[derive(Component)]
struct ChunkMesh {
    index: UVec3,
}

#[derive(Event)]
pub struct FinishedGenerating;

#[derive(Resource)]
pub struct ChunksToGenerateQueue(pub VecDeque<ChunksToGenerateQueueElement>);

pub struct ChunksToGenerateQueueElement {
    pub index: UVec3,
    pub input_data: [bool; BUFFER_LEN_UNCOMPRESSED],
}

pub(crate) struct DigTerrainPlugin;
impl Plugin for DigTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelInteractionPlugin)
            .add_plugins(GpuReadbackPlugin)
            .add_event::<FinishedGenerating>()
            .insert_resource(ChunksToGenerateQueue(VecDeque::new()))
            .add_systems(Update, (handle_voxel_changes, update_mesh));
    }
}

pub fn spawn_terrain(mut chunks_manager: ChunksManager) {
    chunks_manager.create_chunks(UVec3::new(3, 1, 3), VOXEL_SCALE);
}

fn handle_voxel_changes(
    mut set: ParamSet<(Query<Entity, Changed<VoxelChunk>>, ChunksManager)>,
    mut queue: ResMut<ChunksToGenerateQueue>,
) {
    let changed: Vec<Entity> = set.p0().iter().collect();
    for chunk_entity in changed.iter() {
        let manager = set.p1();
        let chunk = manager.get_chunk(*chunk_entity);
        let data = manager.get_chunk_surrounded(chunk.index);
        queue.0.push_back(ChunksToGenerateQueueElement {
            index: chunk.index,
            input_data: data,
        });
    }
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_chunk_r: EventReader<ChunkMeshGenerated>,
    chunks_manager: ChunksManager,
    terrain_q: Query<(Entity, &Mesh3d, &ChunkMesh)>,
) {
    for ev in mesh_chunk_r.read() {
        let scale = VOXEL_SCALE;
        let mesh = ev.mesh.clone().scaled_by(Vec3::splat(scale));
        let collider = Collider::trimesh_from_mesh(&mesh).unwrap();

        if let Some((entity, mesh_handle, _)) = terrain_q
            .iter()
            .find(|(_, _, chunk)| chunk.index == ev.index)
        {
            meshes.insert(mesh_handle, mesh);
            commands.entity(entity).insert(collider);
        } else {
            let size = chunks_manager.get_amount();
            let mut offset = (size.as_vec3() * CHUNK_WIDTH as f32 * VOXEL_SCALE) / 2.;
            offset.y *= 2.;
            commands
                .spawn((
                    Transform::from_translation(
                        (ev.index * CHUNK_WIDTH as u32).as_vec3() * VOXEL_SCALE - offset,
                    ),
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.34, 0.2, 0.2),
                        perceptual_roughness: 1.,
                        ..default()
                    })),
                    ChunkMesh { index: ev.index },
                    RigidBody::Static,
                    collider,
                ))
                .observe(
                    |trigger: Trigger<Pointer<Move>>,
                     q_windows: Query<&Window, With<PrimaryWindow>>,
                     mut commands: Commands| {
                        if !q_windows.single().cursor_options.visible {
                            commands.remove_resource::<PointerPosition>();
                            return;
                        }
                        let Some(pos) = trigger.hit.position else {
                            return;
                        };
                        commands.insert_resource(PointerPosition(pos))
                    },
                )
                .observe(|_: Trigger<Pointer<Out>>, mut commands: Commands| {
                    commands.remove_resource::<PointerPosition>();
                });
        }
    }
}
