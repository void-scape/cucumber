#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

struct Uniforms {
    resolution: vec2f,
    intensity: f32,
    branches: f32,
    color: vec3f,
    origin: vec2f,
    targetp: vec2f,
    maxWidth: f32,
};

@group(2) @binding(0) var<uniform> uniforms: Uniforms;

// Hash function for randomness
fn hash(p: f32) -> f32 {
    var p2 = fract(p * 0.1031);
    p2 *= p2 + 33.33;
    p2 *= p2 + p2;
    return fract(p2);
}

fn hash2(p: vec2f) -> f32 {
    var p3 = fract(vec2f(p.x * p.y, p.x + p.y));
    p3 += dot(p3, p3.yx + 33.33);
    return fract((p3.x + p3.y) * p3.x);
}

// 2D noise function
fn noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    // Cubic interpolation
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(
        mix(hash2(i + vec2f(0.0, 0.0)), 
            hash2(i + vec2f(1.0, 0.0)), u.x),
        mix(hash2(i + vec2f(0.0, 1.0)), 
            hash2(i + vec2f(1.0, 1.0)), u.x),
        u.y
    );
}

// Calculate distance to a lightning segment
fn sdfLine(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Generate a lightning bolt from origin to targetp
fn lightningBolt(uv: vec2f, seed: f32, time: f32) -> f32 {
    let origin = uniforms.origin;
    let targetp = uniforms.targetp;
    
    var segmentCount = 10;
    var displacement = 0.1;
    var result = 0.0;
    var baseThickness = 0.003 * uniforms.intensity;
    
    // Calculate the main bolt direction and length
    var direction = normalize(targetp - origin);
    var totalLength = distance(targetp, origin);
    var segLength = totalLength / f32(segmentCount);
    
    // Apply the maxWidth constraint if needed
    var maxWidth = uniforms.maxWidth;
    if (maxWidth <= 0.0) {
        maxWidth = 1.0; // Default value if not set properly
    }
    
    // Calculate the displacement limit based on maxWidth
    var maxDisplacement = maxWidth / 2.0;
    displacement = min(displacement, maxDisplacement / totalLength);
    
    // Generate the main bolt
    var segStart = origin;
    
    for (var i = 0; i < segmentCount; i++) {
        let noiseOffset = vec2f(
            hash(seed + f32(i) * 0.1 + time * 0.2) * 2.0 - 1.0,
            hash(seed + f32(i) * 0.1 + 0.5 + time * 0.2) * 2.0 - 1.0
        );
        
        var segEnd: vec2f;
        if (i == segmentCount - 1) {
            segEnd = targetp; // Last segment ends at the targetp
        } else {
            let normalDir = vec2f(-direction.y, direction.x);
            var offset = normalDir * noiseOffset.x * displacement * totalLength * (f32(i) / f32(segmentCount));
            
            // Limit the offset to respect maxWidth
            let offsetMagnitude = length(offset);
            if (offsetMagnitude > maxDisplacement) {
                offset = normalize(offset) * maxDisplacement;
            }
            
            segEnd = segStart + direction * segLength + offset;
        }
        
        let dist = sdfLine(uv, segStart, segEnd);
        let thickness = baseThickness;
        let glow = smoothstep(thickness, 0.0, dist);
        result = max(result, glow);
        
        // Generate branches randomly
        if (hash(seed + f32(i) * 10.0 + 3.1) < 0.3 && i > 0 && i < segmentCount - 1 && uniforms.branches > 0.1) {
            // Branch direction
            let branchAngle = 0.3 + 0.4 * hash(seed + f32(i) * 5.0 + time * 0.1);
            let rot = mat2x2f(
                cos(branchAngle), -sin(branchAngle),
                sin(branchAngle), cos(branchAngle)
            );
            let branchDir = rot * direction;
            
            // Branch length
            let branchLength = segLength * (0.3 + 0.7 * hash(seed + f32(i) * 2.5));
            let branchEnd = segStart + branchDir * branchLength;
            
            let branchDist = sdfLine(uv, segStart, branchEnd);
            let branchThickness = thickness * 0.6; // Thinner branches
            let branchGlow = smoothstep(branchThickness, 0.0, branchDist);
            result = max(result, branchGlow * 0.7); // Less bright branches
        }
        
        segStart = segEnd;
    }
    
    // Add time-based flicker
    let flicker = 0.8 + 0.2 * sin(time * 15.0 + hash(seed) * 10.0);
    
    return result * flicker;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4f {
    let uv = input.uv;
    let aspectRatio = uniforms.resolution.x / uniforms.resolution.y;
    let normalizedUV = vec2f(uv.x * aspectRatio, uv.y);
    
    let seedVariation = floor(globals.time * 2.0) * 0.1;
    
    // Generate main lightning
    var lightning = lightningBolt(normalizedUV, seedVariation, globals.time);
    
    // Add some secondary lightning bolts for visual interest
    if (uniforms.branches > 0.5) {
        let offset1 = vec2f(0.03, -0.02) * hash2(vec2f(seedVariation, globals.time));
        let offset2 = vec2f(-0.02, 0.04) * hash2(vec2f(globals.time, seedVariation));
        
        lightning += lightningBolt(normalizedUV, seedVariation + 10.0, globals.time + 0.3) * 0.3;
        lightning += lightningBolt(normalizedUV, seedVariation + 20.0, globals.time + 0.7) * 0.2;
    }
    
    // Color the lightning
    let lightningColor = uniforms.color;
    let glowColor = mix(lightningColor, vec3f(1.0), 0.6); // Core is whiter
    
    // Add glow effect
    let glow = smoothstep(0.0, 1.0, lightning) * 0.5;
    let glowRadius = min(0.04 * uniforms.intensity, uniforms.maxWidth * 0.5);
    
    // Final color composition
    var finalColor = vec3f(0.0);
    finalColor += lightning * glowColor; // Bright core
    finalColor += glow * lightningColor; // Medium glow
    
    // Add some background atmospheric effect
    let backgroundNoise = noise(normalizedUV * 3.0 + globals.time * 0.1) * 0.03;
    let backgroundGlow = max(0.0, lightning * 0.1) * lightningColor;
    finalColor += backgroundNoise * backgroundGlow;
    
    // Calculate alpha value based on lightning intensity for transparency
    let alpha = smoothstep(0.0, 0.05, lightning + glow);
    
    return vec4f(finalColor, alpha);
}
