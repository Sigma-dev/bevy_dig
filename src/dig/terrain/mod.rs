use std::collections::VecDeque;

use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    prelude::*,
    render::{render_resource::BufferUsages, storage::ShaderStorageBuffer},
    window::PrimaryWindow,
};
use interaction::{VoxelInteractionPlugin, VoxelPointerPosition};

use crate::{
    generation::{ChunkMeshGenerated, GpuReadbackPlugin, TerrainData, BUFFER_LEN},
    voxel::VoxelChunk,
};

mod interaction;

pub const VOXEL_SCALE: f32 = 0.25;

#[derive(Resource, Debug)]
pub struct VoxelData(VoxelChunk);

#[derive(Component)]
struct ChunkMesh {
    index: UVec3,
}

#[derive(Resource)]
pub struct ChunksToGenerateQueue(VecDeque<ChunksToGenerateQueueElement>);

pub struct ChunksToGenerateQueueElement {
    pub index: UVec3,
    pub input_data: [bool; BUFFER_LEN],
}

pub(crate) struct DigTerrainPlugin;
impl Plugin for DigTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelInteractionPlugin)
            .add_plugins(GpuReadbackPlugin)
            .insert_resource(ChunksToGenerateQueue(VecDeque::new()))
            .add_systems(Update, (handle_queue, handle_voxel_changes, update_mesh));
    }
}

pub fn spawn_terrain(commands: &mut Commands) {
    commands.insert_resource(VoxelData(VoxelChunk::full()))
}

fn handle_queue(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut queue: ResMut<ChunksToGenerateQueue>,
) {
    let Some(element) = queue.0.pop_front() else {
        return;
    };
    let vec: Vec<u32> = element
        .input_data
        .iter()
        .map(|a| if *a { 1 } else { 0 })
        .collect();
    let mut input_buffer = ShaderStorageBuffer::from(vec);
    input_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let handle = buffers.add(input_buffer);
    commands.insert_resource(TerrainData(handle));
}

fn handle_voxel_changes(
    maybe_voxels: Option<Res<VoxelData>>,
    mut queue: ResMut<ChunksToGenerateQueue>,
) {
    let Some(voxels) = maybe_voxels else { return };
    if voxels.is_changed() {
        queue.0.push_back(ChunksToGenerateQueueElement {
            index: UVec3::ZERO,
            input_data: voxels.0.raw(),
        });
    }
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_chunk_r: EventReader<ChunkMeshGenerated>,
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
            commands
                .spawn((
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
                            commands.remove_resource::<VoxelPointerPosition>();
                            return;
                        }
                        let Some(pos) = trigger.hit.position else {
                            return;
                        };
                        commands.insert_resource(VoxelPointerPosition::from_world_pos(pos))
                    },
                )
                .observe(|_: Trigger<Pointer<Out>>, mut commands: Commands| {
                    commands.remove_resource::<VoxelPointerPosition>();
                });
        }
    }
}
