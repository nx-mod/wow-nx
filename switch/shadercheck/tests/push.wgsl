// The shape of wgpu-core's dispatch validation shader, which crashed device creation on the console (2026-10-23):
// a push constant, a storage buffer read and a storage buffer written.
struct OffsetPc { inner: u32 }
var<push_constant> offset: OffsetPc;
@group(0) @binding(0) var<storage, read_write> dst: array<u32, 6>;
@group(1) @binding(0) var<storage, read> src: array<u32>;

@compute @workgroup_size(1)
fn main() {
    let s = vec3(src[offset.inner], src[offset.inner + 1], src[offset.inner + 2]);
    if s.x > 65535u || s.y > 65535u || s.z > 65535u {
        dst = array<u32, 6>(0u, 0u, 0u, 0u, 0u, 0u);
    } else {
        dst = array<u32, 6>(s.x, s.y, s.z, s.x, s.y, s.z);
    }
}
