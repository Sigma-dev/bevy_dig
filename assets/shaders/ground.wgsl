#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct MyExtendedMaterial {
    quantize_steps: u32,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutputa {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) uv_b: vec2<f32>,
        @location(4) world_tangent: vec4<f32>,
        @location(5) color: vec4<f32>,
        @location(6) @interpolate(flat) instance_index: u32,
        @location(7) @interpolate(flat) visibility_range_dither: i32,
}

//@vertex
//fn vertex(vertex: Vertex) -> VertexOutput {
    //var out: VertexOutput; 
   // out.model_position = vertex.position;
   // out.clip_position = mesh_position_local_to_clip(
  //      get_world_from_local(vertex.instance_index),
  //      vec4<f32>(vertex.position, 1.0),
//    );
//    return out;
//}

@fragment
fn fragment(
    input: VertexOutputa,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let vertex_output = VertexOutput(
    input.clip_position,
    input.world_position,
    input.world_normal,
    input.instance_index,
    );
    let height = input.clip_position.y;
    var color = vec4<f32>(0.4, .2, 0.1, 1.);
    if (height > 500.) {
        color = vec4<f32>(0.1, .8, 0.1, 1.);
    }

    var pbr_input = pbr_input_from_standard_material(vertex_output, is_front);
    pbr_input.material.base_color = color;
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}