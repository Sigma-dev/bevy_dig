#import bevy_pbr::{
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
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
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

fn csin(x: f32) -> f32 {
    return sin(x) + (sin(x * 2) / 2.) + (sin(x * 4.) / 4.);
}

fn height_noise(x: f32, z: f32, amplitude: f32, freq: f32) -> f32 {
    return (csin(x * freq) + csin(z * freq)) * amplitude;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var color =  vec4<f32>(1., 0.5, 0.5, 1.);
    if in.world_position.y > -0.15 + height_noise(in.world_position.x, in.world_position.z, 0.1, 0.5) {
       color = vec4<f32>(0.1, 1., 0.1, 1.);
    }

    pbr_input.material.base_color = alpha_discard(pbr_input.material, color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}