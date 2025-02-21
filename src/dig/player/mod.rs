use avian3d::prelude::*;
use bevy::prelude::*;
use camera::{FpsCamera, FpsCameraPlugin};
use movement::*;

use crate::indexed_camera::IndexedCamera;

pub mod camera;
mod movement;

pub struct DigPlayerPlugin;
impl Plugin for DigPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerMovementPlugin, FpsCameraPlugin));
    }
}

pub fn spawn_player(commands: &mut Commands) {
    let player = commands
        .spawn((
            PlayerMovement::new(100., 10., 10.),
            RigidBody::Kinematic,
            Mass(70.),
            Collider::capsule(0.15, 1.2),
            Transform::from_xyz(5., 20., 5.),
            LockedAxes::ROTATION_LOCKED,
            GroundFriction(0.1),
            HoverSpring::new(1.5, 0.95, 150.),
            KinematicGravity(15.),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        ))
        .id();
    commands.entity(player).with_child((
        IndexedCamera::new(0),
        FpsCamera::new(0.1),
        Transform::from_xyz(0.0, 0.6, 0.0),
        RayCaster::new(Vec3::ZERO, -Dir3::Z)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([player]))
            .with_max_hits(1)
            .with_max_distance(10.),
    ));
}
