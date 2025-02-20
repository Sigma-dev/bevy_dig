use avian3d::PhysicsPlugins;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    render::{render_resource::BufferUsages, storage::ShaderStorageBuffer},
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use generation::{
    convert_booleans_to_buffer, ChunkMeshGenerated, GpuReadbackPlugin, BUFFER_LEN, CHUNK_WIDTH,
};
use interaction::VoxelInteractionPlugin;
use std::collections::VecDeque;
use voxel::VoxelChunk;

mod generation;
mod interaction;
mod voxel;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 42.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                    },
                    enabled: true,
                    ..default()
                },
            },
            GpuReadbackPlugin,
            MeshPickingPlugin,
            PhysicsPlugins::default(),
            VoxelInteractionPlugin,
            DefaultEditorCamPlugins,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(VoxelPointerPosition(None))
        .insert_resource(ChunksToGenerateQueue(VecDeque::new()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (one_time, update_mesh, handle_voxel_changes, handle_queue),
        )
        .run();
}

#[derive(Resource, Debug)]
pub struct TerrainData(Handle<ShaderStorageBuffer>);

#[derive(Resource, Debug)]
pub struct VoxelData(VoxelChunk);

#[derive(Resource)]
pub struct VoxelPointerPosition(Option<Vec3>);

#[derive(Resource)]
pub struct ChunksToGenerateQueue(VecDeque<ChunksToGenerateQueueElement>);

pub struct ChunksToGenerateQueueElement {
    pub index: UVec3,
    pub input_data: [bool; BUFFER_LEN],
}

#[derive(Component)]
struct ChunkMesh {
    index: UVec3,
}

fn setup(mut commands: Commands) {
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
}

fn one_time(mut commands: Commands, time: Res<Time>, maybe_data: Option<Res<VoxelData>>) {
    if time.elapsed_secs() > 0.5 && maybe_data.is_none() {
        commands.insert_resource(VoxelData(VoxelChunk::full())) //Forced to delay creation by a delay because it doesn't work reliably otherwise
    }
}

fn handle_queue(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut queue: ResMut<ChunksToGenerateQueue>,
) {
    let Some(element) = queue.0.pop_front() else {
        return;
    };
    let vec = convert_booleans_to_buffer(&element.input_data.to_vec());
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
    terrain_q: Query<(&Mesh3d, &ChunkMesh)>,
) {
    for ev in mesh_chunk_r.read() {
        if let Some((mesh, _)) = terrain_q.iter().find(|(_, chunk)| chunk.index == ev.index) {
            meshes.insert(mesh, ev.mesh.clone());
        } else {
            commands
                .spawn((
                    Mesh3d(meshes.add(ev.mesh.clone())),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.34, 0.2, 0.2),
                        perceptual_roughness: 1.,
                        ..default()
                    })),
                    ChunkMesh { index: ev.index },
                    Transform::from_translation(-Vec3::splat(CHUNK_WIDTH as f32) / 2.),
                ))
                .observe(
                    |trigger: Trigger<Pointer<Move>>,
                     mut voxel_pos: ResMut<VoxelPointerPosition>| {
                        let Some(pos) = trigger.hit.position else {
                            return;
                        };
                        voxel_pos.0 = Some(
                            pos + Vec3::new(
                                CHUNK_WIDTH as f32 / 2.,
                                CHUNK_WIDTH as f32 / 2.,
                                CHUNK_WIDTH as f32 / 2.,
                            ),
                        );
                    },
                )
                .observe(
                    |_: Trigger<Pointer<Out>>, mut voxel_pos: ResMut<VoxelPointerPosition>| {
                        voxel_pos.0 = None;
                    },
                );
        }
    }
}
