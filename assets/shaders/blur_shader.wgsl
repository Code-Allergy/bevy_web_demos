@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var sampler: sampler;

@fragment
fn fragment_main(@location(0) frag_coord: vec2<f32>) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0);
    var offset: vec2<f32> = vec2<f32>(1.0 / textureDimensions(texture, 0));

    // Sample surrounding pixels for blur effect
    color += textureSample(texture, sampler, frag_coord + offset * vec2(1.0, 0.0));
    color += textureSample(texture, sampler, frag_coord + offset * vec2(-1.0, 0.0));
    color += textureSample(texture, sampler, frag_coord + offset * vec2(0.0, 1.0));
    color += textureSample(texture, sampler, frag_coord + offset * vec2(0.0, -1.0));

    return color / 5.0; // Average the colors
}