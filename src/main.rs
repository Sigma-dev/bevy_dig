use avian3d::{prelude::SleepingPlugin, PhysicsPlugins};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use bevy_steam_p2p::SteamP2PPlugin;
use dig::DigPlugin;
use indexed_camera::{IndexedCamera, IndexedCameraPlugin};
use voxel::chunks_manager::ChunksInfo;

mod dig;
mod generation;
mod indexed_camera;
mod voxel;

fn main() {
    // simulate_shader::run_simulation();
    App::new()
        .add_plugins(SteamP2PPlugin)
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
            MeshPickingPlugin,
            PhysicsPlugins::default()
                .build()
                .disable::<SleepingPlugin>(),
            DefaultEditorCamPlugins,
            IndexedCameraPlugin,
            DigPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Resource)]
struct PlayerSpawned;

pub fn setup(mut commands: Commands) {
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

    commands.spawn((
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        EditorCam::default(),
        Transform::from_translation(Vec3::splat(50.)).looking_at(Vec3::ZERO, Vec3::Y),
        IndexedCamera::new(1),
        PointLight {
            intensity: 100.,
            ..default()
        },
    ));
}
