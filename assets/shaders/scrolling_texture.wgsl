#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> uv_offset: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4f {
    return textureSample(texture, texture_sampler, mesh.uv + vec2f(0, uv_offset));
}
