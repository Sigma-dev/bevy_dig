use bevy::{
    asset::RenderAssetUsages,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::BufferUsages,
        storage::ShaderStorageBuffer,
    },
    text::FontSmoothing,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use generation::{convert_booleans_to_buffer, GpuReadbackPlugin, CHUNK_WIDTH};
use simulate_shader::run_simulation;
use voxel::VoxelChunk;

mod generation;
mod simulate_shader;
mod voxel;

fn main() {
    //let data = run_simulation();

    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                    },
                    enabled: true,
                    ..default()
                },
            },
            GpuReadbackPlugin,
            MeshPickingPlugin,
            DefaultEditorCamPlugins,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TerrainVertices(Vec::new(), Vec::new(), Vec::new()))
        .insert_resource(VoxelPointerPosition(None))
        .insert_resource(VoxelPointerSize(20.))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_mesh, handle_inputs, handle_voxel_changes))
        .add_systems(Update, one_time)
        //.add_systems(Update, draw_simulation)
        .run();
}

#[derive(Resource, Debug)]
pub struct TerrainVertices(Vec<Vec3>, Vec<usize>, Vec<Vec3>);

#[derive(Resource, Debug)]
pub struct TerrainData(Handle<ShaderStorageBuffer>);

#[derive(Resource, Debug)]
pub struct VoxelData(VoxelChunk);

#[derive(Resource)]
pub struct VoxelPointerPosition(Option<Vec3>);

#[derive(Resource)]
pub struct VoxelPointerSize(f32);

fn handle_inputs(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    voxel_pos: Res<VoxelPointerPosition>,
    maybe_voxel_data: Option<ResMut<VoxelData>>,
    mut voxel_size: ResMut<VoxelPointerSize>,
    keys: Res<ButtonInput<KeyCode>>,
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
            pos - Vec3::new(
                CHUNK_WIDTH as f32 / 2.,
                CHUNK_WIDTH as f32 / 2.,
                CHUNK_WIDTH as f32 / 2.,
            ),
        ),
        voxel_size.0,
        Color::WHITE,
    );
    if mouse_buttons.just_pressed(MouseButton::Left) {
        voxel_data.0.dig_hole(pos, voxel_size.0);
    }
    if keys.just_pressed(KeyCode::KeyB) {
        voxel_data.0.build_sphere(pos, voxel_size.0);
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        voxel_size.0 -= 5.;
    }
    if keys.just_pressed(KeyCode::KeyW) {
        voxel_size.0 += 5.;
    }
}

fn one_time(mut commands: Commands, time: Res<Time>, maybe_data: Option<Res<VoxelData>>) {
    if time.elapsed_secs() > 0.01 && maybe_data.is_none() {
        commands.insert_resource(VoxelData(VoxelChunk::full()))
    }
}

fn handle_voxel_changes(
    mut commands: Commands,
    maybe_voxels: Option<Res<VoxelData>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let Some(voxels) = maybe_voxels else { return };
    if !voxels.is_changed() {
        return;
    }
    println!("yep");
    let vec = convert_booleans_to_buffer(&voxels.0.raw().to_vec());
    let mut input_buffer = ShaderStorageBuffer::from(vec);
    /*  let mut input_buffer =
    ShaderStorageBuffer::from(convert_booleans_to_buffer(&&make_sphere_buffer(1.))); */
    input_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let handle = buffers.add(input_buffer);
    commands.insert_resource(TerrainData(handle));
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(),
        Transform::from_translation(Vec3::splat(50.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

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

    commands
        .spawn((
            Mesh3d(meshes.reserve_handle()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.34, 0.2, 0.2),
                perceptual_roughness: 1.,
                ..default()
            })),
            Terrain,
            Transform::from_translation(-Vec3::splat(CHUNK_WIDTH as f32) / 2.),
        ))
        .observe(
            |trigger: Trigger<Pointer<Move>>, mut voxel_pos: ResMut<VoxelPointerPosition>| {
                let Some(pos) = trigger.hit.position else {
                    return;
                };
                voxel_pos.0 = Some(
                    pos + Vec3::new(
                        CHUNK_WIDTH as f32 / 2.,
                        CHUNK_WIDTH as f32 / 2.,
                        CHUNK_WIDTH as f32 / 2.,
                    ),
                );
            },
        )
        .observe(
            |_trigger: Trigger<Pointer<Out>>, mut voxel_pos: ResMut<VoxelPointerPosition>| {
                voxel_pos.0 = None;
            },
        );
}

#[derive(Component)]
struct Terrain;

fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_vertices: Res<TerrainVertices>,
    mut terrain_q: Query<&mut Mesh3d, With<Terrain>>,
    mut gizmos: Gizmos,
) {
    if !terrain_vertices.is_changed() || terrain_vertices.0.len() == 0 {
        return;
    }
    for terrain in terrain_q.iter() {
        //meshes.insert(&terrain.0, create_terrain_mesh(&terrain_vertices.0));
        meshes.insert(
            &terrain.0,
            create_terrain_mesh(&terrain_vertices.1, &terrain_vertices.2),
        );
    }
}

fn create_terrain_mesh(indices: &Vec<usize>, uniques: &Vec<Vec3>) -> Mesh {
    /*   let triangles: Vec<[f32; 3]> = terrain_vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let indices = terrain_vertices
        .iter()
        .enumerate()
        .map(|(i, _)| i as u32)
        .collect();*/
    let indices_u32: Vec<u32> = indices.iter().map(|i| *i as u32).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, uniques.clone())
    .with_inserted_indices(Indices::U32(indices_u32))
    .with_computed_normals();
    mesh
}
