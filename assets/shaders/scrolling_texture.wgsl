#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> uv_offset: f32;
@group(2) @binding(3) var<uniform> alpha: f32;
@group(2) @binding(4) var<uniform> alpha_effect: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4f {
    var c = textureSample(texture, texture_sampler, mesh.uv + vec2f(0, uv_offset));

    // Original alpha calculation
    let original_alpha = c.a * alpha;
    
    // Modified alpha with y-dependent decrease
    let modified_alpha = c.a * (alpha * (1.0 - (mesh.uv.y - 0.)));
    
    // Mix between original and modified alpha based on strength parameter
    c.a = mix(original_alpha, modified_alpha, alpha_effect);

    return c;
}
