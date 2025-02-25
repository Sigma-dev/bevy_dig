use avian3d::{
    math::Scalar,
    prelude::{RigidBody, Rotation, ShapeHits},
};
use bevy::prelude::*;

use crate::dig::player::kcc::movement;

use super::{KCCGravity, KinematicCharacterController};

pub fn kcc_input_plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (velocity_dampening, update_grounded, movement_input).chain(),
    );
}

// Marker component for whether or not we're currently grounded.
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// System that handles player movement and camera rotation based on input
///
/// This system processes player actions and updates the character's movement and camera
/// orientation. It handles horizontal movement, jumping, and camera rotation using mouse input.
pub fn movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(
        &mut KinematicCharacterController,
        Has<Grounded>,
        &mut Transform,
        Option<&mut KCCGravity>,
    )>,
    time: Res<Time>,
) {
    // Early return if we can't get the player or camera
    let Ok((mut kcc, grounded, mut player_transform, mut gravity)) = player_query.get_single_mut()
    else {
        return;
    };

    update_player_movement(
        &keyboard,
        &mut kcc,
        grounded,
        &player_transform,
        gravity.as_deref_mut(),
    );
}

/// Updates the player's movement based on input
fn update_player_movement(
    keyboard: &Res<ButtonInput<KeyCode>>,
    kcc: &mut KinematicCharacterController,
    grounded: bool,
    player_transform: &Transform,
    gravity: Option<&mut KCCGravity>,
) {
    let mut movement = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyA) {
        movement.x = -1.;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement.x = 1.;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        movement.y = 1.;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement.y = -1.;
    }

    let direction = player_transform
        .rotation
        .mul_vec3(Vec3::new(movement.x, 0.0, -movement.y))
        .normalize_or_zero()
        * 10.0;

    if movement != Vec2::ZERO {
        kcc.velocity.x = direction.x;
        kcc.velocity.z = direction.z;
    }

    // Handle jumping through gravity system
    if keyboard.pressed(KeyCode::Space) && grounded {
        if let Some(gravity) = gravity {
            gravity.current_velocity = Vec3::Y * 5.0;
        }
    }
}
/*
/// Updates the camera and player rotation based on mouse input
fn update_camera_rotation(
    action_state: &ActionState<PlayerActions>,
    camera_transform: &mut Transform,
    player_transform: &mut Transform,
    delta_time: f32,
) {
    let sensitivity = Vec2::new(0.12, 0.10);
    let mouse_delta = action_state.axis_pair(&PlayerActions::Camera) * delta_time * sensitivity;
    let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    pitch = (pitch + -mouse_delta.y).clamp(-1.54, 1.54);
    yaw -= mouse_delta.x;

    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    player_transform.rotation = Quat::from_rotation_y(yaw);
}
 */
pub fn velocity_dampening(mut query: Query<&mut KinematicCharacterController>, _time: Res<Time>) {
    for mut kcc in query.iter_mut() {
        kcc.velocity.x *= 0.9;
        kcc.velocity.z *= 0.9;
    }
}

fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &ShapeHits, &Rotation), (With<KCCGravity>, With<RigidBody>)>,
) {
    let _ = (45.0 as Scalar).to_radians();
    for (entity, hits, _) in &mut query {
        let is_grounded = hits.iter().any(|hit| true && hit.entity != entity);

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}
