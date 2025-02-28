use avian3d::{prelude::SleepingPlugin, PhysicsPlugins};
use bevy::{
    core::FrameCount,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use dig::{
    player::spawn_player,
    terrain::{spawn_terrain, FinishedGenerating},
    DigPlugin,
};
use indexed_camera::{IndexedCamera, IndexedCameraPlugin};
use voxel::chunks_manager::ChunksManager;

mod dig;
mod generation;
mod indexed_camera;
mod voxel;

fn main() {
    // simulate_shader::run_simulation();
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
        .add_systems(Update, (delayed_setup, handle_player_spawn))
        .run();
}

#[derive(Resource)]
struct PlayerSpawned;

fn setup(mut commands: Commands) {
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

fn delayed_setup(chunks_manager: ChunksManager, frame_count: Res<FrameCount>) {
    if frame_count.0 == 20 {
        //Forced to delay creation by a delay because it doesn't work reliably otherwise
        spawn_terrain(chunks_manager);
    }
}

fn handle_player_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generation_r: EventReader<FinishedGenerating>,
    maybe_player_spawned: Option<Res<PlayerSpawned>>,
) {
    let read = generation_r.read();
    if read.len() > 0 && maybe_player_spawned.is_none() {
        commands.insert_resource(PlayerSpawned);
        spawn_player(&mut commands, &mut meshes, &mut materials);
    }
}
