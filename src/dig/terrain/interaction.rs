use avian3d::prelude::{Collider, Mass, RayCaster, RayHits, RigidBody};
use bevy::prelude::*;

use crate::dig::player::camera::FpsCamera;

use super::{VoxelData, VOXEL_SCALE};

#[derive(Resource)]
pub struct VoxelPointerSize(f32);

#[derive(Resource)]
pub struct VoxelPointerPosition {
    world_pos: Vec3,
    voxel_pos: Vec3,
}

impl VoxelPointerPosition {
    pub fn from_world_pos(world_pos: Vec3) -> VoxelPointerPosition {
        VoxelPointerPosition {
            world_pos,
            voxel_pos: (world_pos) * (1. / VOXEL_SCALE) + Vec3::splat(-2.),
        }
    }
}

pub struct VoxelInteractionPlugin;
impl Plugin for VoxelInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelPointerSize(5.)).add_systems(
            Update,
            (
                modify_voxels,
                spawn_sphere,
                modify_pointer_size,
                handle_fps_pointer,
            ),
        );
    }
}

fn modify_voxels(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    maybe_voxel_data: Option<ResMut<VoxelData>>,
    voxel_pos: Option<Res<VoxelPointerPosition>>,
    voxel_size: Res<VoxelPointerSize>,
    mut gizmos: Gizmos,
) {
    let Some(mut voxel_data) = maybe_voxel_data else {
        return;
    };
    let Some(pos) = voxel_pos else {
        return;
    };
    gizmos.sphere(
        Isometry3d::from_translation(pos.world_pos),
        voxel_size.0 * VOXEL_SCALE,
        Color::WHITE,
    );
    if mouse_buttons.just_pressed(MouseButton::Left) {
        voxel_data.0.dig_hole(pos.voxel_pos, voxel_size.0);
    }
    if keys.just_pressed(KeyCode::KeyB) {
        voxel_data.0.build_sphere(pos.voxel_pos, voxel_size.0);
    }
}

fn spawn_sphere(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    voxel_pos: Option<Res<VoxelPointerPosition>>,
) {
    let Some(pos) = voxel_pos else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyR) {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Collider::sphere(1.),
            Mass(1.),
            RigidBody::Dynamic,
            Transform::from_translation(pos.world_pos + Vec3::Y * 10.),
        ));
    }
}

fn handle_fps_pointer(
    raycast_q: Query<(&GlobalTransform, &RayCaster, &RayHits), With<FpsCamera>>,
    mut commands: Commands,
) {
    for (gt, raycast, hits) in raycast_q.iter() {
        let Some(hit) = hits.iter().next() else {
            continue;
        };
        let pos: Vec3 = gt.translation() + raycast.origin + gt.forward() * hit.distance;
        commands.insert_resource::<VoxelPointerPosition>(VoxelPointerPosition::from_world_pos(pos));
    }
}

fn modify_pointer_size(keys: Res<ButtonInput<KeyCode>>, mut voxel_size: ResMut<VoxelPointerSize>) {
    if keys.pressed(KeyCode::KeyQ) {
        voxel_size.0 = (voxel_size.0 - 0.2).max(1.);
    }
    if keys.pressed(KeyCode::KeyE) {
        voxel_size.0 += 0.2;
    }
}
