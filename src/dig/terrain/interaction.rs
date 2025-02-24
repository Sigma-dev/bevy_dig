use avian3d::prelude::{Collider, Mass, RayCaster, RayHits, RigidBody};
use bevy::prelude::*;

use crate::{dig::player::camera::FpsCamera, voxel::chunks_manager::ChunksManager};

use super::VOXEL_SCALE;

#[derive(Resource)]
pub struct VoxelPointerSize(f32);

#[derive(Resource)]
pub struct PointerPosition(pub Vec3);

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
    mut chunks_manager: ChunksManager,
    pointer_pos: Option<Res<PointerPosition>>,
    voxel_size: Res<VoxelPointerSize>,
    mut gizmos: Gizmos,
) {
    let Some(pos) = pointer_pos else {
        return;
    };
    gizmos.sphere(
        Isometry3d::from_translation(pos.0),
        voxel_size.0,
        Color::WHITE,
    );
    if mouse_buttons.just_pressed(MouseButton::Left) {
        chunks_manager.dig_sphere(pos.0, voxel_size.0);
    }
    if keys.just_pressed(KeyCode::KeyB) {
        chunks_manager.build_sphere(pos.0, voxel_size.0);
    }
}

fn spawn_sphere(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pointer_pos: Option<Res<PointerPosition>>,
) {
    let Some(pos) = pointer_pos else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyR) {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Collider::sphere(1.),
            Mass(1.),
            RigidBody::Dynamic,
            Transform::from_translation(pos.0 + Vec3::Y * 10.),
        ));
    }
}

fn handle_fps_pointer(
    raycast_q: Query<(&GlobalTransform, &RayCaster, &RayHits), With<FpsCamera>>,
    mut commands: Commands,
) {
    for (gt, raycast, hits) in raycast_q.iter() {
        let Some(hit) = hits.iter().next() else {
            commands.remove_resource::<PointerPosition>();
            continue;
        };
        let pos: Vec3 = gt.translation() + raycast.origin + gt.forward() * hit.distance;
        commands.insert_resource::<PointerPosition>(PointerPosition(pos));
    }
}

fn modify_pointer_size(keys: Res<ButtonInput<KeyCode>>, mut voxel_size: ResMut<VoxelPointerSize>) {
    if keys.pressed(KeyCode::KeyQ) {
        voxel_size.0 = (voxel_size.0 - 0.2).max(2. * VOXEL_SCALE);
    }
    if keys.pressed(KeyCode::KeyE) {
        voxel_size.0 += 0.2;
    }
}
