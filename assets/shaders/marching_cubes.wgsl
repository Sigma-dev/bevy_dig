const INTERNAL_CHUNK_WIDTH: u32 = 31;
const MAX_VERTICES_PER_VOXEL: u32 = 12;
const CHUNK_WIDTH: u32 = INTERNAL_CHUNK_WIDTH + 2u;
const INPUT_LENGTH = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH / 32;
const OUTPUT_LENGTH = INPUT_LENGTH * MAX_VERTICES_PER_VOXEL;
@group(0) @binding(0) var<storage, read_write> input_data: array<u32, INPUT_LENGTH>;
@group(0) @binding(1) var<storage, read_write> output_data: array<vec4<f32>, OUTPUT_LENGTH>;
@group(0) @binding(2) var<storage, read> triangles_table: array<array<i32, 16>, 256>;

fn is_voxel_empty(pos: vec3<u32>) -> bool {
    let index = pos.x + pos.y * CHUNK_WIDTH + pos.z * CHUNK_WIDTH * CHUNK_WIDTH;
    let u32_index = index / 32u;
    let bit_index = index % 32u;
    let compressed_u32 = input_data[u32_index];

    return ((compressed_u32 >> bit_index) & 1u) == 0u;
}

fn index_to_output_index(coords: vec3<u32>) -> u32 {
    return (coords.x + coords.y * CHUNK_WIDTH + coords.z * CHUNK_WIDTH * CHUNK_WIDTH) * MAX_VERTICES_PER_VOXEL;
}

fn get_middle(a: vec3<u32>, b: vec3<u32>) -> vec3<f32> {
    let a_f32 = vec3<f32>(f32(a.x), f32(a.y), f32(a.z));
    let b_f32 = vec3<f32>(f32(b.x), f32(b.y), f32(b.z));

    return vec3<f32>(
        (a_f32.x + b_f32.x) * 0.5,
        (a_f32.y + b_f32.y) * 0.5,
        (a_f32.z + b_f32.z) * 0.5
    );
}

fn corner_index_to_coordinates(index: vec3<u32>, corner_index: u32) -> vec3<u32> {
    const offset = array<vec3<u32>, 8>(
        vec3<u32>(0, 0, 0),
        vec3<u32>(1, 0, 0),
        vec3<u32>(1, 1, 0),
        vec3<u32>(0, 1, 0),
        vec3<u32>(0, 0, 1),
        vec3<u32>(1, 0, 1),
        vec3<u32>(1, 1, 1),
        vec3<u32>(0, 1, 1)
    );
    return index + offset[corner_index];
}

@compute @workgroup_size(4, 4, 4)
fn main(
    @builtin(global_invocation_id) index: vec3<u32>,
) {
    if output_data[0].w == -1. {
        output_data[0].w = 42.0;
    }
    let coordinates = array<vec3<u32>, 8>(
        index,
        index + vec3<u32>(1, 0, 0),
        index + vec3<u32>(1, 1, 0),
        index + vec3<u32>(0, 1, 0),
        index + vec3<u32>(0, 0, 1),
        index + vec3<u32>(1, 0, 1),
        index + vec3<u32>(1, 1, 1),
        index + vec3<u32>(0, 1, 1),
    );
    
    var corners = array<bool, 8>();
    for (var i: u32 = 0; i < 8; i++) {
       corners[i] = is_voxel_empty(coordinates[i]);
    }

    var cube_index: u32 = 0;
    for (var i: u32 = 0; i < 8; i++) {
        if (corners[i]) {
            cube_index = cube_index | (1u << i);
        }
    }

    let edges = triangles_table[cube_index];
    for (var i: u32 = 0; i < 16 && edges[i] != -1; i += 3u) {
        for (var j: u32 = 0; j < 3; j++) {
            let edge = edges[i + j];
            let corners = edge_index_to_conter_index[edge];
            let p1 = corner_index_to_coordinates(index,  corners[0]);
            let p2 = corner_index_to_coordinates(index, corners[1]);
            let middle = get_middle(p1, p2);
            let coor = index_to_output_index(index);
            output_data[coor + i + j] = vec4<f32>(middle.x, middle.y, middle.z, 0.0);
        }
    }
}

const edge_index_to_conter_index = array<array<u32, 2>, 12>(
    array<u32, 2>(0, 1),
    array<u32, 2>(1, 2),
    array<u32, 2>(2, 3),
    array<u32, 2>(3, 0),
    array<u32, 2>(4, 5),
    array<u32, 2>(5, 6),
    array<u32, 2>(6, 7),
    array<u32, 2>(7, 4),
    array<u32, 2>(0, 4),
    array<u32, 2>(1, 5),
    array<u32, 2>(2, 6),
    array<u32, 2>(3, 7)
);