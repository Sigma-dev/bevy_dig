#import bevy_pbr::forward_io::VertexOutput

fn cclamp(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let horizon_color = vec4<f32>(0.8, 0.8, 0.8, 1.);
    let top_color = vec4<f32>(0.5, 0.8, 1., 1.);
    let mix = mix(horizon_color, top_color, (mesh.uv.y * 2.6) + 1.);
    return vec4<f32>(cclamp(mix.x), cclamp(mix.y), cclamp(mix.z), 1.);
}