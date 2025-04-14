const INTERNAL_CHUNK_WIDTH: u32 = 31;
const MAX_VERTICES_PER_VOXEL: u32 = 12;
const CHUNK_WIDTH: u32 = INTERNAL_CHUNK_WIDTH + 2u;
const INPUT_LENGTH = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH / 32;
const OUTPUT_LENGTH = INPUT_LENGTH * MAX_VERTICES_PER_VOXEL;
@group(0) @binding(0) var<storage, read_write> input_data: array<u32, INPUT_LENGTH>;
@group(0) @binding(1) var<storage, read_write> output_data: array<vec4<f32>, OUTPUT_LENGTH>;
@group(0) @binding(2) var<storage, read> triangles_table: array<array<i32, 16>, 256>;

@compute @workgroup_size(4, 4, 4)
fn main(
    @builtin(global_invocation_id) index: vec3<u32>,
) {
    let edges = triangles_table[0];
     for (var i: u32 = 0; i <= 12; i += 3u) {
        if (edges[i] != -1) {
            break;
        }
        let edge = edges[i];
        output_data[0] = vec4<f32>(0.);
    }
}