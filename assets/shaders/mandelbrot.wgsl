#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Params {
  center: vec2f,
  zoom: f32,   
  aspectRatio: f32,
  maxIterations: u32,
  colorShift: f32,  
}

@group(2) @binding(0) var<uniform> params: Params;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4f {
  let c = params.center + (mesh.uv - 0.5) * vec2f(params.aspectRatio, 1.0) / params.zoom;
  var z = vec2f(0.0, 0.0);
  var z2 = vec2f(0.0, 0.0);
  var i: u32 = 0u;
  
  while (i < params.maxIterations && z2.x + z2.y < 4.0) {
    z = vec2f(z2.x - z2.y, 2.0 * z.x * z.y) + c;
    z2 = vec2f(z.x * z.x, z.y * z.y);
    i++;
  }
  
  if (i == params.maxIterations) {
    return vec4f(0.0, 0.0, 0.0, 1.0);
  }
  
  let smoothed = f32(i) - log2(log2(z2.x + z2.y)) + 4.0;
  let normalized = smoothed / f32(params.maxIterations);
  
  let t = normalized + params.colorShift;
  let r = 0.5 + 0.5 * sin(3.1415 * t);
  let g = 0.5 + 0.5 * sin(3.1415 * t + 2.0);
  let b = 0.5 + 0.5 * sin(3.1415 * t + 4.0);
  
  return vec4f(r, g, b, 1.0);
}
