#version 450 core
#extension GL_ARB_compute_shader : require
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

struct ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX {
    mat3x3 balance;
    vec3 saturation;
    vec3 contrast;
    vec3 gamma;
    vec3 gain;
    vec3 lift;
    vec2 midtone_range;
    float exposure;
    float hue;
    float post_saturation;
};
struct ViewX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX {
    mat4x4 clip_from_world;
    mat4x4 unjittered_clip_from_world;
    mat4x4 world_from_clip;
    mat4x4 world_from_view;
    mat4x4 view_from_world;
    mat4x4 clip_from_view;
    mat4x4 view_from_clip;
    vec3 world_position;
    float exposure;
    vec4 viewport;
    vec4 main_pass_viewport;
    vec4 frustum[6];
    ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX color_grading;
    float mip_bias;
    uint frame_count;
};
layout(binding = 0) uniform sampler2DShadow _group_0_binding_0_cs;

layout(binding = 0,r16f) writeonly uniform image2D _group_0_binding_1_cs;

layout(binding = 1,r16f) writeonly uniform image2D _group_0_binding_2_cs;

layout(binding = 2,r16f) writeonly uniform image2D _group_0_binding_3_cs;

layout(binding = 3,r16f) writeonly uniform image2D _group_0_binding_4_cs;

layout(binding = 4,r16f) writeonly uniform image2D _group_0_binding_5_cs;

layout(std140, binding = 0) uniform ViewX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX_block_0Compute { ViewX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX _group_1_binding_2_cs; };

shared float previous_mip_depth[8][8];


float weighted_average(float depth0_, float depth1_, float depth2_, float depth3_) {
    float effect_radius = ((0.75 * 0.5) * 1.457);
    float falloff_range = (0.615 * effect_radius);
    float falloff_from = (effect_radius * 0.385);
    float falloff_mul = (-1.0 / falloff_range);
    float falloff_add = ((falloff_from / falloff_range) + 1.0);
    float min_depth = min(min(depth0_, depth1_), min(depth2_, depth3_));
    float weight0_ = clamp((((depth0_ - min_depth) * falloff_mul) + falloff_add), 0.0, 1.0);
    float weight1_ = clamp((((depth1_ - min_depth) * falloff_mul) + falloff_add), 0.0, 1.0);
    float weight2_ = clamp((((depth2_ - min_depth) * falloff_mul) + falloff_add), 0.0, 1.0);
    float weight3_ = clamp((((depth3_ - min_depth) * falloff_mul) + falloff_add), 0.0, 1.0);
    float weight_total = (((weight0_ + weight1_) + weight2_) + weight3_);
    return (((((weight0_ * depth0_) + (weight1_ * depth1_)) + (weight2_ * depth2_)) + (weight3_ * depth3_)) / weight_total);
}

void main() {
    uvec3 global_id = gl_GlobalInvocationID;
    uvec3 local_id = gl_LocalInvocationID;
    ivec2 base_coordinates = ivec2(global_id.xy);
    ivec2 pixel_coordinates0_ = (base_coordinates * 2);
    ivec2 pixel_coordinates1_ = (pixel_coordinates0_ + ivec2(1, 0));
    ivec2 pixel_coordinates2_ = (pixel_coordinates0_ + ivec2(0, 1));
    ivec2 pixel_coordinates3_ = (pixel_coordinates0_ + ivec2(1, 1));
    vec4 _e20 = _group_1_binding_2_cs.viewport;
    vec2 depths_uv = (vec2(pixel_coordinates0_) / _e20.zw);
    vec4 depths = textureGatherOffset(_group_0_binding_0_cs, vec2(depths_uv), ivec2(1, 1), 0);
    imageStore(_group_0_binding_1_cs, pixel_coordinates0_, vec4(depths.w, 0.0, 0.0, 0.0));
    imageStore(_group_0_binding_1_cs, pixel_coordinates1_, vec4(depths.z, 0.0, 0.0, 0.0));
    imageStore(_group_0_binding_1_cs, pixel_coordinates2_, vec4(depths.x, 0.0, 0.0, 0.0));
    imageStore(_group_0_binding_1_cs, pixel_coordinates3_, vec4(depths.y, 0.0, 0.0, 0.0));
    float _e57 = weighted_average(depths.w, depths.z, depths.x, depths.y);
    imageStore(_group_0_binding_2_cs, base_coordinates, vec4(_e57, 0.0, 0.0, 0.0));
    previous_mip_depth[local_id.x][local_id.y] = _e57;
    memoryBarrierShared();
    barrier();
    if (all(equal((local_id.xy % uvec2(2u)), uvec2(0u)))) {
        float depth0_1 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 0u)];
        float depth1_1 = previous_mip_depth[(local_id.x + 1u)][(local_id.y + 0u)];
        float depth2_1 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 1u)];
        float depth3_1 = previous_mip_depth[(local_id.x + 1u)][(local_id.y + 1u)];
        float _e117 = weighted_average(depth0_1, depth1_1, depth2_1, depth3_1);
        imageStore(_group_0_binding_3_cs, (base_coordinates / ivec2(2)), vec4(_e117, 0.0, 0.0, 0.0));
        previous_mip_depth[local_id.x][local_id.y] = _e117;
    }
    memoryBarrierShared();
    barrier();
    if (all(equal((local_id.xy % uvec2(4u)), uvec2(0u)))) {
        float depth0_2 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 0u)];
        float depth1_2 = previous_mip_depth[(local_id.x + 2u)][(local_id.y + 0u)];
        float depth2_2 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 2u)];
        float depth3_2 = previous_mip_depth[(local_id.x + 2u)][(local_id.y + 2u)];
        float _e179 = weighted_average(depth0_2, depth1_2, depth2_2, depth3_2);
        imageStore(_group_0_binding_4_cs, (base_coordinates / ivec2(4)), vec4(_e179, 0.0, 0.0, 0.0));
        previous_mip_depth[local_id.x][local_id.y] = _e179;
    }
    memoryBarrierShared();
    barrier();
    if (all(equal((local_id.xy % uvec2(8u)), uvec2(0u)))) {
        float depth0_3 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 0u)];
        float depth1_3 = previous_mip_depth[(local_id.x + 4u)][(local_id.y + 0u)];
        float depth2_3 = previous_mip_depth[(local_id.x + 0u)][(local_id.y + 4u)];
        float depth3_3 = previous_mip_depth[(local_id.x + 4u)][(local_id.y + 4u)];
        float _e241 = weighted_average(depth0_3, depth1_3, depth2_3, depth3_3);
        imageStore(_group_0_binding_5_cs, (base_coordinates / ivec2(8)), vec4(_e241, 0.0, 0.0, 0.0));
        return;
    } else {
        return;
    }
}

