use avian3d::{
    math::{Quaternion, Vector},
    prelude::*,
};
use bevy::{color::palettes::css, prelude::*};
use camera::{FpsCamera, FpsCameraPlugin};
use kcc::{
    plugin, KCCFloorDetection, KCCGravity, KCCGrounded, KCCSlope, KinematicCharacterController,
};
use movement::*;

use crate::indexed_camera::IndexedCamera;

pub mod camera;
mod kcc;
mod movement;

pub struct DigPlayerPlugin;
impl Plugin for DigPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerMovementPlugin, FpsCameraPlugin, plugin));
    }
}

/* pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let radius = 0.15;
    let player = commands
        .spawn((
            PlayerMovement::new(100., 10., 10.),
            RigidBody::Kinematic,
            Mass(70.),
            Collider::capsule(0.15, 1.2),
            Transform::from_xyz(0., 2., 0.),
            LockedAxes::ROTATION_LOCKED,
            GroundFriction(0.1),
            HoverSpring::new(1.5, 0.95, 150.),
            KinematicGravity(15.),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Mesh3d(meshes.add(Capsule3d::new(radius, 1.8))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
        ))
        .id();
    commands.entity(player).with_child((
        IndexedCamera::new(0),
        FpsCamera::new(0.1),
        Transform::from_xyz(0.0, 0.6, 0.0),
        RayCaster::new(Vec3::ZERO, -Dir3::Z)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([player]))
            .with_max_hits(1)
            .with_max_distance(30.),
    ));
}
 */

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let player = commands
        .spawn((
            RigidBody::Kinematic,
            KCCGravity::default(),
            ShapeCaster::new(
                Capsule3d::new(0.4, 0.8),
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            ),
            KinematicCharacterController::default(),
            KCCGrounded::default(),
            KCCFloorDetection::default(),
            KCCSlope::default(),
            Mesh3d(meshes.add(Capsule3d {
                radius: 0.4,
                half_length: 0.4,
            })),
            MeshMaterial3d(materials.add(Color::from(css::DARK_CYAN))),
            LockedAxes::ROTATION_LOCKED,
            Name::new("CurrentPlayer"),
            Transform::from_xyz(0.0, 1.5, 0.0),
        ))
        .id();

    commands.entity(player).with_child((
        IndexedCamera::new(0),
        FpsCamera::new(0.1),
        Transform::from_xyz(0.0, 0.6, 0.0),
        RayCaster::new(Vec3::ZERO, -Dir3::Z)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([player]))
            .with_max_hits(1)
            .with_max_distance(30.),
    ));
}
