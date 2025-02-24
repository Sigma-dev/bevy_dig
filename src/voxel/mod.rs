use bevy::{math::FloatPow, prelude::*};

use crate::generation::{CHUNK_DATA, CHUNK_WIDTH};

pub mod chunks_manager;

#[derive(Component, Debug)]
pub struct VoxelChunk {
    pub index: UVec3,
    voxels: [bool; CHUNK_DATA as usize],
}

impl VoxelChunk {
    pub fn new(index: UVec3, voxels: [bool; CHUNK_DATA as usize]) -> VoxelChunk {
        VoxelChunk { index, voxels }
    }

    pub fn full(index: UVec3) -> VoxelChunk {
        VoxelChunk::new(index, [true; CHUNK_DATA as usize])
    }

    pub fn raw(&self) -> [bool; CHUNK_DATA as usize] {
        self.voxels
    }

    pub fn get_chunk_width(&self) -> usize {
        CHUNK_WIDTH
    }

    pub fn get_index(&self, pos: UVec3) -> usize {
        let width = self.get_chunk_width() as u32;
        (pos.x + pos.y * width + pos.z * width as u32 * width) as usize
    }

    pub fn get_pos(&self, index: usize) -> UVec3 {
        let width = self.get_chunk_width();
        UVec3::new(
            (index % width) as u32,
            ((index / width) % width) as u32,
            (index / (width * width)) as u32,
        )
    }

    pub fn set_voxel(&mut self, pos: UVec3, value: bool) {
        self.voxels[self.get_index(pos)] = value;
    }

    pub fn set_sphere(&mut self, pos: Vec3, size: f32, state: bool) {
        let size_squared = size.squared();
        for i in 0..self.voxels.len() {
            let voxel_pos = self.get_pos(i);

            if voxel_pos.as_vec3().distance_squared(pos) < size_squared {
                self.set_voxel(voxel_pos, state);
            }
        }
    }
}
