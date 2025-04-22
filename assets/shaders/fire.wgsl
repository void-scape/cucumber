#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> uv_offset: f32;

// @fragment
// fn fragment(mesh: VertexOutput) -> @location(0) vec4f {
//     let color = textureSample(texture, texture_sampler, mesh.uv + vec2f(0, uv_offset));
//     let y = clamp(mesh.uv.y - 0.2, 0.0, 1.0);
//     let x = sin(mesh.uv.x * 3.1415);
//     let w = color.x * clamp(y * y * x, 0.0, 1.0);
//     return vec4f(w, w, w, 1);
// }

// Helper function to convert linear value to radial (0->0, 0.5->1, 1->0)
fn linear_to_radial(t: f32) -> f32 {
    return sin(3.14159 * t);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fire parameters - adjust these for different fire effects
    let fire_speed = 0.5;
    let fire_intensity = 1.6;
    let fire_scale = vec2<f32>(0.5, 1.0);
    
    // Colors for the fire gradient
    let color_dark = vec3<f32>(0.0, 0.0, 0.0);         // Black/transparent base
    let color_bottom = vec3<f32>(0.7, 0.0, 0.0);       // Deep red
    let color_middle = vec3<f32>(0.9, 0.4, 0.0);       // Orange
    let color_top = vec3<f32>(1.0, 0.8, 0.2);          // Yellow/white

    // Create sliding UV coordinates for the noise
    var uv = in.uv;
    uv.y = uv.y + globals.time * fire_speed;
    
    // Scale UVs to control the noise pattern size
    uv = uv * fire_scale;
    
    let noise_value = textureSample(texture, texture_sampler, uv).r;
    let uv2 = in.uv * 1.3 + vec2<f32>(0.5, globals.time * fire_speed * 1.2);
    let noise_value2 = textureSample(texture, texture_sampler, uv2).r;
    
    // Combine noise layers
    let combined_noise = noise_value * 0.7 + noise_value2 * 0.3;
    
    // Use UV y coordinate to create the fire shape (fading toward top)
    let fire_shape = 1.0 - in.uv.y;
    
    var intensity = linear_to_radial(fire_shape) * fire_intensity;
    intensity = max(intensity - (1.0 - combined_noise), 0.0);
    
    // Apply the fire shape mask to get sharper edges at the bottom
    var fire_mask = smoothstep(0.0, 0.3, combined_noise - (1.0 - fire_shape * 2.0));
    
    // Create color gradient based on height
    var color = vec3<f32>(0.0);
    let pos_y = in.uv.y;
    
    color = mix(color_middle, color_bottom, pos_y);
    
    // Apply final intensity to the color
    color = color * intensity;

    let x = linear_to_radial(in.uv.x);
    let w = smoothstep(0.4, 0.6, pow(x, 4.0));
    color *= w;
    
    // Add some glow effect
    color = pow(color, vec3<f32>(0.8));
    
    // Alpha based on intensity for proper blending
    let alpha = min(intensity * fire_mask * 1.5, 1.0);
    
    return vec4<f32>(color * 2.0, alpha);
}
