use avian3d::prelude::{Collider, Mass, RigidBody};
use bevy::prelude::*;

use crate::{generation::CHUNK_WIDTH, VoxelData, VoxelPointerPosition};

#[derive(Resource)]
pub struct VoxelPointerSize(f32);

pub struct VoxelInteractionPlugin;
impl Plugin for VoxelInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelPointerSize(20.))
            .add_systems(Update, (modify_voxels, spawn_sphere, modify_pointer_size));
    }
}

fn modify_voxels(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    maybe_voxel_data: Option<ResMut<VoxelData>>,
    voxel_pos: Res<VoxelPointerPosition>,
    voxel_size: Res<VoxelPointerSize>,
    mut gizmos: Gizmos,
) {
    let Some(mut voxel_data) = maybe_voxel_data else {
        return;
    };
    let Some(pos) = voxel_pos.0 else {
        return;
    };
    gizmos.sphere(
        Isometry3d::from_translation(
            pos.1
                - Vec3::new(
                    CHUNK_WIDTH as f32 / 2.,
                    CHUNK_WIDTH as f32 / 2.,
                    CHUNK_WIDTH as f32 / 2.,
                ),
        ),
        voxel_size.0,
        Color::WHITE,
    );
    if mouse_buttons.just_pressed(MouseButton::Left) {
        voxel_data.0.dig_hole(pos.1, voxel_size.0);
    }
    if keys.just_pressed(KeyCode::KeyB) {
        voxel_data.0.build_sphere(pos.1, voxel_size.0);
    }
}

fn spawn_sphere(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    voxel_pos: Res<VoxelPointerPosition>,
) {
    let Some(pos) = voxel_pos.0 else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyS) {
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

fn modify_pointer_size(keys: Res<ButtonInput<KeyCode>>, mut voxel_size: ResMut<VoxelPointerSize>) {
    if keys.just_pressed(KeyCode::KeyQ) {
        voxel_size.0 -= 2.;
    }
    if keys.just_pressed(KeyCode::KeyW) {
        voxel_size.0 += 2.;
    }
}
