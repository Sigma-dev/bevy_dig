use bevy::{math::FloatPow, prelude::*};

use crate::generation::{BUFFER_LEN, CHUNK_WIDTH};

#[derive(Debug)]
pub struct VoxelChunk {
    voxels: [bool; BUFFER_LEN as usize],
}

impl VoxelChunk {
    pub fn new(voxels: [bool; BUFFER_LEN as usize]) -> VoxelChunk {
        VoxelChunk { voxels }
    }

    pub fn empty() -> VoxelChunk {
        VoxelChunk::new([false; BUFFER_LEN as usize])
    }

    pub fn full() -> VoxelChunk {
        VoxelChunk::new([true; BUFFER_LEN as usize])
    }

    pub fn raw(&self) -> [bool; BUFFER_LEN as usize] {
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

    pub fn get_voxel(&self, pos: UVec3) -> bool {
        self.voxels[self.get_index(pos)]
    }

    pub fn set_voxel(&mut self, pos: UVec3, value: bool) {
        self.voxels[self.get_index(pos)] = value;
    }

    pub fn remove_voxel(&mut self, pos: UVec3) {
        self.set_voxel(pos, false);
    }

    pub fn add_voxel(&mut self, pos: UVec3) {
        self.set_voxel(pos, true);
    }

    pub fn dig_hole(&mut self, pos: Vec3, size: f32) {
        let size_squared = size.squared();
        for i in 0..self.voxels.len() {
            let voxel_pos = self.get_pos(i);

            if voxel_pos.as_vec3().distance_squared(pos) < size_squared {
                self.remove_voxel(voxel_pos);
            }
        }
    }

    pub fn build_sphere(&mut self, pos: Vec3, size: f32) {
        let size_squared = size.squared();
        for i in 0..self.voxels.len() {
            let voxel_pos = self.get_pos(i);

            if voxel_pos.as_vec3().distance_squared(pos) < size_squared {
                self.add_voxel(voxel_pos);
            }
        }
    }
}
