use std::collections::VecDeque;

use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    window::PrimaryWindow,
};
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

pub(crate) struct DigTerrainPlugin;
impl Plugin for DigTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelInteractionPlugin)
            .add_plugins(
                (
                    GpuReadbackPlugin,
                    MaterialPlugin::<ExtendedMaterial<StandardMaterial, GroundMaterial>>::default(),
                ), //  MaterialPlugin::<GroundMaterial>::default(),
            )
            .add_event::<FinishedGenerating>()
            .insert_resource(ChunksToGenerateQueue(VecDeque::new()))
            .add_systems(Update, (handle_voxel_changes, update_mesh));
    }
}

pub fn spawn_terrain(mut chunks_manager: ChunksManager) {}

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
    //    mut ground_materials: ResMut<Assets<GroundMaterial>>,
    mut ground2_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GroundMaterial>>>,
    mut mesh_chunk_r: EventReader<ChunkMeshGenerated>,
    chunks_manager: ChunksManager,
    terrain_q: Query<(Entity, &Mesh3d, &ChunkMesh)>,
) {
    for ev in mesh_chunk_r.read() {
        let scale = VOXEL_SCALE;
        let mesh = ev.mesh.clone().scaled_by(Vec3::splat(scale));
        let collider = Collider::trimesh_from_mesh(&mesh).unwrap();

        /* let _ground_handle = ground_materials.add(GroundMaterial {
            alpha_mode: AlphaMode::Blend,
        }); */
        let _material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.2, 0.2),
            perceptual_roughness: 1.,
            ..default()
        });
        let _ground2_handle = ground2_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::WHITE,
                opaque_render_method: OpaqueRendererMethod::Auto,
                ..Default::default()
            },
            extension: GroundMaterial {},
        });

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
                    MeshMaterial3d(_ground2_handle),
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

/* #[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct GroundMaterial {
    alpha_mode: AlphaMode,
}

impl Material for GroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
} */

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct GroundMaterial {}

impl MaterialExtension for GroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }
}
