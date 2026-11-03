// The title screen's background: the launcher's Mono banner, as the Fabric mod shows it.
struct Params {
    size: vec2<f32>,
    image: vec2<f32>,
    time: f32,
    picture: f32,
    // 1 keeps a picture's bottom edge on screen (a banner's ground), 0 centres it.
    anchor: f32,
    pad: f32,
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var picture: texture_2d<f32>;
@group(0) @binding(2) var picture_sampler: sampler;

@vertex fn vs_backdrop(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corners = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(corners[index], 0.0, 1.0);
}

@fragment fn fs_backdrop(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // CSS `cover`: filled edge to edge, the rest cut off.
    let scale = max(params.size.x / params.image.x, params.size.y / params.image.y);
    let shown = params.image * scale;
    let offset = (params.size - shown) * vec2(0.5, mix(0.5, 1.0, params.anchor));
    return vec4(textureSample(picture, picture_sampler, (position.xy - offset) / shown).rgb, 1.0);
}
