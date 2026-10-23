
struct VsOut { @builtin(position) pos: vec4f, @location(0) color: vec3f, @location(1) uv: vec2f };

@vertex fn vs_triangle(@location(0) pos: vec2f, @location(1) color: vec3f) -> VsOut {
    return VsOut(vec4f(pos, 0.0, 1.0), color, vec2f(0.0));
}
@fragment fn fs_triangle(e: VsOut) -> @location(0) vec4f {
    return vec4f(e.color, 1.0);
}

@group(0) @binding(0) var<uniform> tint: vec4f;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@vertex fn vs_quad(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> VsOut {
    var corners = array(vec2f(0.0, 0.0), vec2f(1.0, 0.0), vec2f(0.0, 1.0), vec2f(1.0, 0.0), vec2f(1.0, 1.0), vec2f(0.0, 1.0));
    let c = corners[v];
    // Instance 1 lands on the right half; instance 0 would land on the left.
    let x = -0.8 + f32(i) * 1.0 + c.x * 0.6;
    let y = -0.4 + c.y * 0.8;
    return VsOut(vec4f(x, y, 0.0, 1.0), vec3f(0.0), vec2f(c.x, 1.0 - c.y));
}
@fragment fn fs_quad(e: VsOut) -> @location(0) vec4f {
    return textureSample(tex, samp, e.uv) * tint;
}
