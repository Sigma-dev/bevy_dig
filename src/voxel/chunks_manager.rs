use bevy::{
    ecs::system::SystemParam,
    math::{
        bounding::{Aabb3d, IntersectsVolume},
        Vec3A,
    },
    prelude::*,
};

use crate::{
    dig::terrain::VOXEL_SCALE,
    generation::{BUFFER_LEN, CHUNK_DATA, CHUNK_WIDTH},
};

use super::VoxelChunk;

#[derive(SystemParam)]
pub struct ChunksManager<'w, 's> {
    #[doc(hidden)]
    commands: Commands<'w, 's>,
    #[doc(hidden)]
    chunks: Query<'w, 's, &'static mut VoxelChunk>,
    #[doc(hidden)]
    chunks_info: Option<Res<'w, ChunksInfo>>,
}

#[derive(Resource)]
pub struct ChunksInfo {
    amount: UVec3,
}

impl<'w, 's> ChunksManager<'w, 's> {
    pub fn create_chunks(&mut self, amount: UVec3, scale: f32) {
        if amount.x == 0 || amount.y == 0 || amount.z == 0 {
            panic!("Amount should be atleast 1 on all axis");
        }
        self.commands.insert_resource(ChunksInfo { amount });
        for x in 0..amount.x {
            for y in 0..amount.y {
                for z in 0..amount.z {
                    let index = UVec3::new(x, y, z);
                    self.commands.spawn((
                        Transform::from_translation(index.as_vec3() * scale),
                        VoxelChunk::full(index),
                    ));
                }
            }
        }
    }

    pub fn set_sphere(&mut self, world_pos: Vec3, radius: f32, state: bool) {
        let voxel_pos = self.world_pos_to_voxel_pos(world_pos);
        let voxel_radius = Self::world_length_to_voxel_length(radius);
        let operation_bounds = Aabb3d {
            min: (voxel_pos - Vec3::splat(voxel_radius)).into(),
            max: (voxel_pos + Vec3::splat(voxel_radius)).into(),
        };
        for mut chunk in self.chunks.iter_mut() {
            let chunk_min = (chunk.index * CHUNK_WIDTH as u32).as_vec3a();
            let chunk_bounds = Aabb3d {
                min: chunk_min,
                max: chunk_min + (UVec3::splat(CHUNK_WIDTH as u32 - 1)).as_vec3a(),
            };
            if operation_bounds.intersects(&chunk_bounds) {
                let localized_pos = voxel_pos - <Vec3A as Into<Vec3>>::into(chunk_min);
                chunk.set_sphere(localized_pos, voxel_radius, state);
            }
        }
    }

    pub fn get_amount(&self) -> UVec3 {
        self.chunks_info.as_ref().map_or(UVec3::ZERO, |i| i.amount)
    }

    pub fn get_chunk(&self, entity: Entity) -> &VoxelChunk {
        self.chunks.get(entity).unwrap()
    }

    pub fn get_chunk_by_index(&self, index: UVec3) -> Option<&VoxelChunk> {
        self.chunks.iter().find(|c| c.index == index)
    }

    pub fn get_chunk_and_surrounding_data(&self, index: UVec3) -> [bool; CHUNK_DATA * 27] {
        let mut result = [false; CHUNK_DATA * 27];
        for z in 0..3 {
            for y in 0..3 {
                for x in 0..3 {
                    let mut data = [false; CHUNK_DATA];
                    if let Ok(offset_index) =
                        (index.as_ivec3() + IVec3::new(x - 1, y - 1, z - 1)).try_into()
                    {
                        if let Some(chunk) = self.get_chunk_by_index(offset_index) {
                            data = chunk.raw();
                        }
                    }
                    for z2 in 0..CHUNK_WIDTH {
                        for y2 in 0..CHUNK_WIDTH {
                            for x2 in 0..CHUNK_WIDTH {
                                let write_pos = (x2 + x as usize * CHUNK_WIDTH)
                                    + (y2 + y as usize * CHUNK_WIDTH) * CHUNK_WIDTH * 3
                                    + (z2 + z as usize * CHUNK_WIDTH)
                                        * CHUNK_WIDTH
                                        * 3
                                        * CHUNK_WIDTH
                                        * 3;
                                let read_pos =
                                    x2 + y2 * CHUNK_WIDTH + z2 * CHUNK_WIDTH * CHUNK_WIDTH;
                                result[write_pos] = data[read_pos]
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_chunk_surrounded(&self, index: UVec3) -> [bool; BUFFER_LEN] {
        let data = self.get_chunk_and_surrounding_data(index);
        let mut result = [false; BUFFER_LEN];
        for x in 0..(CHUNK_WIDTH + 2) {
            for y in 0..(CHUNK_WIDTH + 2) {
                for z in 0..(CHUNK_WIDTH + 2) {
                    let offset = UVec3::splat(CHUNK_WIDTH as u32 - 1)
                        + UVec3::new(x as u32, y as u32, z as u32);
                    let read_pos = (offset.x
                        + offset.y * CHUNK_WIDTH as u32 * 3
                        + offset.z * (CHUNK_WIDTH * 3 * CHUNK_WIDTH * 3) as u32)
                        as usize;
                    let write_pos =
                        x + y * (CHUNK_WIDTH + 2) + z * (CHUNK_WIDTH + 2) * (CHUNK_WIDTH + 2);
                    result[write_pos] = data[read_pos];
                }
            }
        }
        result
    }

    pub fn dig_sphere(&mut self, world_pos: Vec3, radius: f32) {
        self.set_sphere(world_pos, radius, false);
    }

    pub fn build_sphere(&mut self, world_pos: Vec3, radius: f32) {
        self.set_sphere(world_pos, radius, true);
    }

    fn world_pos_to_voxel_pos(&self, world_pos: Vec3) -> Vec3 {
        let mut middle_offset = self.get_amount().as_vec3() * CHUNK_WIDTH as f32 / 2.;
        middle_offset.y *= 2.;
        let voxel_pos_no_offset = world_pos * (1. / VOXEL_SCALE);
        middle_offset + voxel_pos_no_offset + Vec3::splat(-1.)
    }

    fn world_length_to_voxel_length(world_length: f32) -> f32 {
        world_length * (1. / VOXEL_SCALE)
    }
}
