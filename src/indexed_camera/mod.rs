use bevy::prelude::*;

pub struct IndexedCameraPlugin;
impl Plugin for IndexedCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_cameras);
    }
}

#[derive(Component)]
pub struct IndexedCamera {
    index: u32,
}

impl IndexedCamera {
    pub fn new(index: u32) -> IndexedCamera {
        IndexedCamera { index }
    }
}

fn toggle_cameras(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Camera, &IndexedCamera)>,
) {
    let mut maybe_index = None;

    if keys.just_pressed(KeyCode::Numpad0) {
        maybe_index = Some(0)
    }
    if keys.just_pressed(KeyCode::Numpad1) {
        maybe_index = Some(1)
    }
    if keys.just_pressed(KeyCode::Numpad2) {
        maybe_index = Some(2)
    }
    if keys.just_pressed(KeyCode::Numpad3) {
        maybe_index = Some(3)
    }
    if keys.just_pressed(KeyCode::Numpad4) {
        maybe_index = Some(4)
    }
    if keys.just_pressed(KeyCode::Numpad5) {
        maybe_index = Some(5)
    }
    if keys.just_pressed(KeyCode::Numpad6) {
        maybe_index = Some(6)
    }
    if keys.just_pressed(KeyCode::Numpad7) {
        maybe_index = Some(7)
    }
    if keys.just_pressed(KeyCode::Numpad8) {
        maybe_index = Some(8)
    }
    if keys.just_pressed(KeyCode::Numpad9) {
        maybe_index = Some(9)
    }
    let Some(index) = maybe_index else {
        return;
    };
    for (mut camera, camera_index) in camera_q.iter_mut() {
        let selected = camera_index.index == index;
        camera.is_active = selected;
    }
}
