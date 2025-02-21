use bevy::{
    ecs::system::SystemParam,
    math::{
        bounding::{Aabb3d, IntersectsVolume},
        Vec3A,
    },
    prelude::*,
};

use crate::{dig::terrain::VOXEL_SCALE, generation::CHUNK_WIDTH};

use super::VoxelChunk;

#[derive(SystemParam)]
pub struct ChunksManager<'w, 's> {
    #[doc(hidden)]
    commands: Commands<'w, 's>,
    #[doc(hidden)]
    chunks: Query<'w, 's, &'static mut VoxelChunk>,
}

impl<'w, 's> ChunksManager<'w, 's> {
    pub fn create_chunks(&mut self, amount: UVec3, scale: f32) {
        if amount.x == 0 || amount.y == 0 || amount.z == 0 {
            panic!("Amount should be atleast 1 on all axis");
        }
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
        let voxel_pos = Self::world_pos_to_voxel_pos(world_pos);
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

    pub fn dig_sphere(&mut self, world_pos: Vec3, radius: f32) {
        self.set_sphere(world_pos, radius, false);
    }

    pub fn build_sphere(&mut self, world_pos: Vec3, radius: f32) {
        self.set_sphere(world_pos, radius, true);
    }

    fn world_pos_to_voxel_pos(world_pos: Vec3) -> Vec3 {
        world_pos * (1. / VOXEL_SCALE) + Vec3::splat(-2.)
    }

    fn world_length_to_voxel_length(world_length: f32) -> f32 {
        world_length * (1. / VOXEL_SCALE)
    }
}
