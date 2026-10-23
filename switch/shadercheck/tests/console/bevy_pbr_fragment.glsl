#version 450 core
#extension GL_ARB_shader_storage_buffer_object : require
#extension GL_EXT_texture_shadow_lod : require
struct VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX {
    vec4 position;
    vec4 world_position;
    vec3 world_normal;
    vec2 uv;
    uint instance_index;
};
struct FragmentOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX {
    vec4 color;
};
struct StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX {
    vec4 base_color;
    vec4 emissive;
    vec4 attenuation_color;
    mat3x3 uv_transform;
    vec3 reflectance;
    float perceptual_roughness;
    float metallic;
    float diffuse_transmission;
    float specular_transmission;
    float thickness;
    float ior;
    float attenuation_distance;
    float clearcoat;
    float clearcoat_perceptual_roughness;
    float anisotropy_strength;
    vec2 anisotropy_rotation;
    uint flags;
    float alpha_cutoff;
    float parallax_depth_scale;
    float max_parallax_layer_count;
    float lightmap_exposure;
    uint max_relief_mapping_search_steps;
    uint deferred_lighting_pass_id;
};
struct PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX {
    StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX material;
    vec3 diffuse_occlusion;
    float specular_occlusion;
    vec4 frag_coord;
    vec4 world_position;
    vec3 world_normal;
    vec3 N;
    vec3 V;
    vec3 lightmap_light;
    vec3 clearcoat_N;
    float anisotropy_strength;
    vec3 anisotropy_T;
    vec3 anisotropy_B;
    bool is_orthographic;
    uint flags;
};
struct MeshX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX {
    mat3x4 world_from_local;
    mat3x4 previous_world_from_local;
    mat2x4 local_from_world_transpose_a;
    float local_from_world_transpose_b;
    uint flags;
    uvec2 lightmap_uv_rect;
    uint first_vertex_index;
    uint current_skin_index;
    uint material_and_lightmap_bind_group_slot;
    uint tag;
    uint pad;
};
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
struct DirectionalCascadeX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    mat4x4 clip_from_world;
    float texel_size;
    float far_bound;
};
struct DirectionalLightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    DirectionalCascadeX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX cascades[4];
    vec4 color;
    vec3 direction_to_light;
    uint flags;
    float soft_shadow_size;
    float shadow_depth_bias;
    float shadow_normal_bias;
    uint num_cascades;
    float cascades_overlap_proportion;
    uint depth_texture_base_index;
    uint decal_index;
    float sun_disk_angular_size;
    float sun_disk_intensity;
};
struct LightsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    DirectionalLightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX directional_lights[10];
    vec4 ambient_color;
    uvec4 cluster_dimensions;
    vec4 cluster_factors;
    uint n_directional_lights;
    int spot_light_shadowmap_offset;
    uint ambient_light_affects_lightmapped_meshes;
};
struct FogX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    vec4 base_color;
    vec4 directional_light_color;
    vec3 be;
    float directional_light_exponent;
    vec3 bi;
    uint mode;
};
struct ClusterableObjectX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    vec4 light_custom_data;
    vec4 color_inverse_square_range;
    vec4 position_radius;
    uint flags;
    float shadow_depth_bias;
    float shadow_normal_bias;
    float spot_light_tan_angle;
    float soft_shadow_size;
    float shadow_map_near_z;
    uint decal_index;
    float pad;
};
struct LightProbeX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    mat3x4 light_from_world_transposed;
    int cubemap_index;
    float intensity;
    uint affects_lightmapped_mesh_diffuse;
};
struct LightProbesX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    LightProbeX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX reflection_probes[8];
    LightProbeX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX irradiance_volumes[8];
    int reflection_probe_count;
    int irradiance_volume_count;
    int view_cubemap_index;
    uint smallest_specular_mip_level_for_view;
    float intensity_for_view;
    uint view_environment_map_affects_lightmapped_mesh_diffuse;
};
struct ScreenSpaceReflectionsSettingsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    float perceptual_roughness_threshold;
    float thickness;
    uint linear_steps;
    float linear_march_exponent;
    uint bisection_steps;
    uint use_secant;
};
struct EnvironmentMapUniformX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX {
    mat4x4 transform;
};
struct GlobalsX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DUZ3MN5RGC3DTX {
    float time;
    float delta_time;
    uint frame_count;
};
struct LayerLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX {
    vec3 N;
    vec3 R;
    float NdotV;
    float perceptual_roughness;
    float roughness;
};
struct LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX {
    LayerLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX layers[1];
    vec3 P;
    vec3 V;
    vec3 diffuse_color;
    vec3 F0_;
    vec2 F_ab;
};
struct DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX {
    vec3 H;
    float NdotL;
    float NdotH;
    float LdotH;
};
struct ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX {
    uint first_point_light_index_offset;
    uint first_spot_light_index_offset;
    uint first_reflection_probe_index_offset;
    uint first_irradiance_volume_index_offset;
    uint first_decal_offset;
    uint last_clusterable_object_index_offset;
};
struct SampleBiasX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX {
    float mip_bias;
};
const uint STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 1u;
const uint STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 2u;
const uint STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 4u;
const uint STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 8u;
const uint STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 16u;
const uint STANDARD_MATERIAL_FLAGS_UNLIT_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 32u;
const uint STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUEX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 0u;
const uint STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAPX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 64u;
const uint STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_YX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 128u;
const uint STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITSX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX = 3758096384u;
const float PIX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX = 3.1415927;
const uint POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVEX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX = 2u;
const uint LAYER_BASEX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX = 0u;
const uint MESH_FLAGS_SHADOW_RECEIVER_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX = 536870912u;
const uint POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX = 1u;
const uint DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX = 1u;
const float PI_2X_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX = 6.2831855;
const vec2 SPIRAL_OFFSET_0_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(-0.7071, 0.7071);
const vec2 SPIRAL_OFFSET_1_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(-0.0, -0.875);
const vec2 SPIRAL_OFFSET_2_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(0.5303, 0.5303);
const vec2 SPIRAL_OFFSET_3_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(-0.625, -0.0);
const vec2 SPIRAL_OFFSET_4_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(0.3536, -0.3536);
const vec2 SPIRAL_OFFSET_5_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(-0.0, 0.375);
const vec2 SPIRAL_OFFSET_6_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(-0.1768, -0.1768);
const vec2 SPIRAL_OFFSET_7_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX = vec2(0.125, -0.0);
const float SPOT_SHADOW_TEXEL_SIZEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX = 0.013427734;
const float POINT_SHADOW_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX = 0.003;
const float POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX = 0.5;
const float FRAC_PI_3X_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX = 1.0471976;
const vec3 flip_zX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX = vec3(1.0, 1.0, -1.0);

layout(std430, binding = 4) readonly buffer type_10_block_0Fragment { MeshX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX _group_2_binding_0_fs[]; };

layout(std140, binding = 0) uniform ViewX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX_block_1Fragment { ViewX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX _group_0_binding_0_fs; };

layout(std140, binding = 1) uniform LightsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX_block_2Fragment { LightsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX _group_0_binding_1_fs; };

layout(std430, binding = 0) readonly buffer ClusterableObjectsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX_block_3Fragment {
    ClusterableObjectX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX data[];
} _group_0_binding_8_fs;

layout(std430, binding = 1) readonly buffer ClusterLightIndexListsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX_block_4Fragment {
    uint data[];
} _group_0_binding_9_fs;

layout(std430, binding = 2) readonly buffer ClusterOffsetsAndCountsX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX_block_5Fragment {
    uvec4 data[][2];
} _group_0_binding_10_fs;

layout(binding = 0) uniform samplerCubeArrayShadow _group_0_binding_2_fs;

layout(binding = 1) uniform sampler2DArrayShadow _group_0_binding_5_fs;

layout(std140, binding = 2) uniform GlobalsX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DUZ3MN5RGC3DTX_block_6Fragment { GlobalsX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DUZ3MN5RGC3DTX _group_0_binding_11_fs; };

layout(std140, binding = 7) uniform StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX_block_7Fragment { StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _group_3_binding_0_fs; };

layout(binding = 8) uniform sampler2D _group_3_binding_1_fs;

layout(binding = 9) uniform sampler2D _group_3_binding_3_fs;

layout(binding = 10) uniform sampler2D _group_3_binding_5_fs;

layout(binding = 11) uniform sampler2D _group_3_binding_7_fs;

layout(location = 0) smooth in vec4 _vs2fs_location0;
layout(location = 1) smooth in vec3 _vs2fs_location1;
layout(location = 2) smooth in vec2 _vs2fs_location2;
layout(location = 6) flat in uint _vs2fs_location6;
layout(location = 0) out vec4 _fs2p_location0;

StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX standard_material_newX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX() {
    StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX material = StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(vec4(0.0), vec4(0.0), vec4(0.0), mat3x3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, vec2(0.0), 0u, 0.0, 0.0, 0.0, 0.0, 0u, 0u);
    material.base_color = vec4(1.0, 1.0, 1.0, 1.0);
    material.emissive = vec4(0.0, 0.0, 0.0, 1.0);
    material.perceptual_roughness = 0.5;
    material.metallic = 0.0;
    material.reflectance = vec3(0.5);
    material.diffuse_transmission = 0.0;
    material.specular_transmission = 0.0;
    material.thickness = 0.0;
    material.ior = 1.5;
    material.attenuation_distance = 1.0;
    material.attenuation_color = vec4(1.0, 1.0, 1.0, 1.0);
    material.clearcoat = 0.0;
    material.clearcoat_perceptual_roughness = 0.0;
    material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUEX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX;
    material.alpha_cutoff = 0.5;
    material.parallax_depth_scale = 0.1;
    material.max_parallax_layer_count = 16.0;
    material.max_relief_mapping_search_steps = 5u;
    material.deferred_lighting_pass_id = 1u;
    material.uv_transform = mat3x3(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));
    StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e66 = material;
    return _e66;
}

PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_newX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX() {
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_1 = PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(vec4(0.0), vec4(0.0), vec4(0.0), mat3x3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, vec2(0.0), 0u, 0.0, 0.0, 0.0, 0.0, 0u, 0u), vec3(0.0), 0.0, vec4(0.0), vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), 0.0, vec3(0.0), vec3(0.0), false, 0u);
    StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e2 = standard_material_newX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX();
    pbr_input_1.material = _e2;
    pbr_input_1.diffuse_occlusion = vec3(1.0);
    pbr_input_1.specular_occlusion = 1.0;
    pbr_input_1.frag_coord = vec4(0.0, 0.0, 0.0, 1.0);
    pbr_input_1.world_position = vec4(0.0, 0.0, 0.0, 1.0);
    pbr_input_1.world_normal = vec3(0.0, 0.0, 1.0);
    pbr_input_1.is_orthographic = false;
    pbr_input_1.N = vec3(0.0, 0.0, 1.0);
    pbr_input_1.V = vec3(1.0, 0.0, 0.0);
    pbr_input_1.clearcoat_N = vec3(0.0);
    pbr_input_1.anisotropy_T = vec3(0.0);
    pbr_input_1.anisotropy_B = vec3(0.0);
    pbr_input_1.lightmap_light = vec3(0.0);
    pbr_input_1.flags = 0u;
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e51 = pbr_input_1;
    return _e51;
}

vec2 F_ABX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float perceptual_roughness, float NdotV) {
    vec4 c0_ = vec4(-1.0, -0.0275, -0.572, 0.022);
    vec4 c1_ = vec4(1.0, 0.0425, 1.04, -0.04);
    vec4 r = ((perceptual_roughness * c0_) + c1_);
    float a004_ = ((min((r.x * r.x), exp2((-9.28 * NdotV))) * r.x) + r.y);
    return ((vec2(-1.04, 1.04) * a004_) + r.zw);
}

float perceptualRoughnessToRoughnessX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float perceptualRoughness) {
    float clampedPerceptualRoughness = clamp(perceptualRoughness, 0.089, 1.0);
    return (clampedPerceptualRoughness * clampedPerceptualRoughness);
}

float getDistanceAttenuationX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float distanceSquare, float inverseRangeSquared) {
    float factor = (distanceSquare * inverseRangeSquared);
    float smoothFactor = clamp((1.0 - (factor * factor)), 0.0, 1.0);
    float attenuation = (smoothFactor * smoothFactor);
    return ((attenuation * 1.0) / max(distanceSquare, 0.0001));
}

vec4 compute_specular_layer_values_for_point_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_, uint layer, vec3 V, vec3 light_to_frag, float light_radius, float distance_) {
    float LtFdotR = 0.0;
    vec3 R = input_.layers[layer].R;
    float a_2 = input_.layers[layer].roughness;
    LtFdotR = dot(light_to_frag, R);
    float _e13 = LtFdotR;
    LtFdotR = max(0.0001, _e13);
    float _e16 = LtFdotR;
    vec3 centerToRay = ((_e16 * R) - light_to_frag);
    vec3 closestPoint = (light_to_frag + (centerToRay * clamp((light_radius * inversesqrt(dot(centerToRay, centerToRay))), 0.0, 1.0)));
    float LspecLengthInverse = inversesqrt(dot(closestPoint, closestPoint));
    float a_prime = clamp((a_2 + (light_radius / (2.0 * distance_))), 0.0, 1.0);
    vec3 L_1 = (closestPoint * LspecLengthInverse);
    return vec4(L_1, a_prime);
}

DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX derive_lighting_inputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3 N, vec3 V_1, vec3 L) {
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_1 = DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3(0.0), 0.0, 0.0, 0.0);
    vec3 H = vec3(0.0);
    H = normalize((L + V_1));
    vec3 _e7 = H;
    input_1.H = _e7;
    input_1.NdotL = clamp(dot(N, L), 0.0, 1.0);
    vec3 _e13 = H;
    input_1.NdotH = clamp(dot(N, _e13), 0.0, 1.0);
    vec3 _e17 = H;
    input_1.LdotH = clamp(dot(L, _e17), 0.0, 1.0);
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX _e20 = input_1;
    return _e20;
}

float specular_fix_remapX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float a) {
    float inv_a_sq = ((1.0 - a) * (1.0 - a));
    return (1.0 - (inv_a_sq * inv_a_sq));
}

float D_GGXX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float roughness, float NdotH) {
    float oneMinusNdotHSquared = (1.0 - (NdotH * NdotH));
    float a_3 = (NdotH * roughness);
    float k = (roughness / (oneMinusNdotHSquared + (a_3 * a_3)));
    float d = ((k * k) * 0.31830987);
    return d;
}

float V_SmithGGXCorrelatedX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float roughness_1, float NdotV_1, float NdotL) {
    float a2_ = (roughness_1 * roughness_1);
    float lambdaV = (NdotL * sqrt((((NdotV_1 - (a2_ * NdotV_1)) * NdotV_1) + a2_)));
    float lambdaL = (NdotV_1 * sqrt((((NdotL - (a2_ * NdotL)) * NdotL) + a2_)));
    float v_1 = (0.5 / (lambdaV + lambdaL));
    return v_1;
}

vec3 F_Schlick_vecX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3 f0_, float f90_, float VdotH) {
    return (f0_ + ((vec3(f90_) - f0_) * pow((1.0 - VdotH), 5.0)));
}

vec3 fresnelX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3 f0_1, float LdotH) {
    float f90_2 = clamp(dot(f0_1, vec3(16.5)), 0.0, 1.0);
    vec3 _e6 = F_Schlick_vecX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(f0_1, f90_2, LdotH);
    return _e6;
}

vec3 specular_multiscatterX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float D, float V_2, vec3 F, vec3 F0_, vec2 F_ab, float specular_intensity) {
    vec3 Fr = vec3(0.0);
    Fr = (((specular_intensity * D) * V_2) * F);
    vec3 _e19 = Fr;
    Fr = (_e19 * (vec3(1.0) + (F0_ * ((1.0 / F_ab.x) - 1.0))));
    vec3 _e21 = Fr;
    return _e21;
}

vec3 specularX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_2, inout DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX derived_input, float roughness_2, float specular_intensity_1) {
    float NdotV_3 = input_2.layers[0].NdotV;
    vec3 F0_2 = input_2.F0_;
    float NdotL_1 = derived_input.NdotL;
    float NdotH_1 = derived_input.NdotH;
    float LdotH_1 = derived_input.LdotH;
    float _e15 = D_GGXX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(roughness_2, NdotH_1);
    float _e16 = V_SmithGGXCorrelatedX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(roughness_2, NdotV_3, NdotL_1);
    vec3 _e17 = fresnelX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(F0_2, LdotH_1);
    vec2 _e19 = input_2.F_ab;
    vec3 _e21 = specular_multiscatterX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(_e15, _e16, _e17, F0_2, _e19, specular_intensity_1);
    return _e21;
}

float F_SchlickX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(float f0_2, float f90_1, float VdotH_1) {
    return (f0_2 + ((f90_1 - f0_2) * pow((1.0 - VdotH_1), 5.0)));
}

float Fd_BurleyX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_3, inout DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX derived_input_1) {
    float roughness_3 = input_3.layers[0].roughness;
    float NdotV_4 = input_3.layers[0].NdotV;
    float NdotL_2 = derived_input_1.NdotL;
    float LdotH_2 = derived_input_1.LdotH;
    float f90_3 = (0.5 + (((2.0 * roughness_3) * LdotH_2) * LdotH_2));
    float _e21 = F_SchlickX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(1.0, f90_3, NdotL_2);
    float _e23 = F_SchlickX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(1.0, f90_3, NdotV_4);
    return ((_e21 * _e23) * 0.31830987);
}

vec3 point_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(uint light_id, inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_4, bool enable_diffuse, bool enable_texture) {
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX specular_derived_input = DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3(0.0), 0.0, 0.0, 0.0);
    vec3 specular_light = vec3(0.0);
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX derived_input_2 = DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3(0.0), 0.0, 0.0, 0.0);
    vec3 diffuse = vec3(0.0);
    vec3 color_times_NdotL = vec3(0.0);
    float texture_sample = 1.0;
    vec3 diffuse_color_1 = input_4.diffuse_color;
    vec3 P = input_4.P;
    vec3 N_1 = input_4.layers[0].N;
    vec3 V_5 = input_4.V;
    vec4 _e19 = _group_0_binding_8_fs.data[light_id].position_radius;
    vec3 light_to_frag_1 = (_e19.xyz - P);
    vec3 L_2 = normalize(light_to_frag_1);
    float distance_square = dot(light_to_frag_1, light_to_frag_1);
    float distance_1 = sqrt(distance_square);
    float _e27 = _group_0_binding_8_fs.data[light_id].color_inverse_square_range.w;
    float _e28 = getDistanceAttenuationX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(distance_square, _e27);
    float a_4 = input_4.layers[0].roughness;
    float _e35 = _group_0_binding_8_fs.data[light_id].position_radius.w;
    vec4 _e37 = compute_specular_layer_values_for_point_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(input_4, LAYER_BASEX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX, V_5, light_to_frag_1, _e35, distance_1);
    vec3 L_spec = _e37.xyz;
    float a_prime_1 = _e37.w;
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX _e40 = derive_lighting_inputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(N_1, V_5, L_spec);
    specular_derived_input = _e40;
    float normalizationFactor = (a_4 / a_prime_1);
    float specular_intensity_2 = (normalizationFactor * normalizationFactor);
    float _e44 = specular_fix_remapX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(a_4);
    float brdf_roughness = mix(a_4, a_prime_1, _e44);
    vec3 _e46 = specularX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(input_4, specular_derived_input, brdf_roughness, specular_intensity_2);
    specular_light = _e46;
    float light_radius_1 = _group_0_binding_8_fs.data[light_id].position_radius.w;
    if ((light_radius_1 > 0.0)) {
        float solid_angle = ((light_radius_1 * light_radius_1) / (distance_1 * distance_1));
        float _e57 = specular_derived_input.NdotL;
        float _e59 = specular_derived_input.NdotL;
        vec3 _e65 = specular_light;
        specular_light = (_e65 * clamp((_e57 / max((_e59 + solid_angle), 0.0001)), 0.0, 1.0));
    }
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX _e67 = derive_lighting_inputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(N_1, V_5, L_2);
    derived_input_2 = _e67;
    if (enable_diffuse) {
        float _e70 = Fd_BurleyX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(input_4, derived_input_2);
        diffuse = (diffuse_color_1 * _e70);
    }
    vec3 _e73 = diffuse;
    float _e75 = derived_input_2.NdotL;
    vec3 _e77 = specular_light;
    float _e79 = specular_derived_input.NdotL;
    color_times_NdotL = ((_e73 * _e75) + (_e77 * _e79));
    vec3 _e84 = color_times_NdotL;
    vec4 _e86 = _group_0_binding_8_fs.data[light_id].color_inverse_square_range;
    float _e90 = texture_sample;
    return (((_e84 * _e86.xyz) * _e28) * _e90);
}

vec3 spot_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(uint light_id_1, inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_5, bool enable_diffuse_1) {
    vec3 spot_dir = vec3(0.0);
    float texture_sample_1 = 1.0;
    vec3 _e5 = point_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(light_id_1, input_5, enable_diffuse_1, false);
    float _e11 = _group_0_binding_8_fs.data[light_id_1].light_custom_data.x;
    float _e14 = _group_0_binding_8_fs.data[light_id_1].light_custom_data.y;
    spot_dir = vec3(_e11, 0.0, _e14);
    float _e20 = spot_dir.x;
    float _e22 = spot_dir.x;
    float _e27 = spot_dir.z;
    float _e29 = spot_dir.z;
    spot_dir.y = sqrt(max(0.0, ((1.0 - (_e20 * _e22)) - (_e27 * _e29))));
    uint _e36 = _group_0_binding_8_fs.data[light_id_1].flags;
    if (((_e36 & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVEX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX) != 0u)) {
        float _e43 = spot_dir.y;
        spot_dir.y = -(_e43);
    }
    vec4 _e46 = _group_0_binding_8_fs.data[light_id_1].position_radius;
    vec3 _e49 = input_5.P;
    vec3 light_to_frag_2 = (_e46.xyz - _e49.xyz);
    vec3 _e52 = spot_dir;
    float cd = dot(-(_e52), normalize(light_to_frag_2));
    float _e58 = _group_0_binding_8_fs.data[light_id_1].light_custom_data.z;
    float _e62 = _group_0_binding_8_fs.data[light_id_1].light_custom_data.w;
    float attenuation_1 = clamp(((cd * _e58) + _e62), 0.0, 1.0);
    float spot_attenuation = (attenuation_1 * attenuation_1);
    float _e68 = texture_sample_1;
    return ((_e5 * spot_attenuation) * _e68);
}

vec3 directional_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(uint light_id_2, inout LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX input_6, bool enable_diffuse_2) {
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX derived_input_3 = DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3(0.0), 0.0, 0.0, 0.0);
    vec3 diffuse_1 = vec3(0.0);
    vec3 color = vec3(0.0);
    float texture_sample_2 = 1.0;
    vec3 diffuse_color_2 = input_6.diffuse_color;
    float NdotV_5 = input_6.layers[0].NdotV;
    vec3 N_2 = input_6.layers[0].N;
    vec3 V_6 = input_6.V;
    float roughness_4 = input_6.layers[0].roughness;
    vec3 _e25 = _group_0_binding_1_fs.directional_lights[light_id_2].direction_to_light;
    vec3 L_3 = _e25.xyz;
    DerivedLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX _e27 = derive_lighting_inputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(N_2, V_6, L_3);
    derived_input_3 = _e27;
    if (enable_diffuse_2) {
        float _e30 = Fd_BurleyX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(input_6, derived_input_3);
        diffuse_1 = (diffuse_color_2 * _e30);
    }
    vec3 _e34 = specularX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(input_6, derived_input_3, roughness_4, 1.0);
    vec3 _e35 = diffuse_1;
    float _e38 = derived_input_3.NdotL;
    color = ((_e35 + _e34) * _e38);
    vec4 _e43 = _group_0_binding_1_fs.directional_lights[light_id_2].color;
    float _e45 = texture_sample_2;
    vec3 _e47 = color;
    color = (_e47 * (_e43.xyz * _e45));
    vec3 _e49 = color;
    return _e49;
}

uint view_z_to_z_sliceX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(float view_z, bool is_orthographic) {
    uint z_slice = 0u;
    if (is_orthographic) {
        float _e6 = _group_0_binding_1_fs.cluster_factors.z;
        float _e11 = _group_0_binding_1_fs.cluster_factors.w;
        z_slice = uint(floor(((view_z - _e6) * _e11)));
    } else {
        float _e21 = _group_0_binding_1_fs.cluster_factors.z;
        float _e26 = _group_0_binding_1_fs.cluster_factors.w;
        z_slice = uint((((log(-(view_z)) * _e21) - _e26) + 1.0));
    }
    uint _e31 = z_slice;
    uint _e35 = _group_0_binding_1_fs.cluster_dimensions.z;
    return min(_e31, (_e35 - 1u));
}

uint fragment_cluster_indexX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(vec2 frag_coord, float view_z_1, bool is_orthographic_1) {
    vec4 _e3 = _group_0_binding_0_fs.viewport;
    vec4 _e8 = _group_0_binding_1_fs.cluster_factors;
    uvec2 xy = uvec2(floor(((frag_coord - _e3.xy) * _e8.xy)));
    uint _e15 = view_z_to_z_sliceX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(view_z_1, is_orthographic_1);
    uint _e20 = _group_0_binding_1_fs.cluster_dimensions.x;
    uint _e27 = _group_0_binding_1_fs.cluster_dimensions.z;
    uint _e33 = _group_0_binding_1_fs.cluster_dimensions.w;
    return min(((((xy.y * _e20) + xy.x) * _e27) + _e15), (_e33 - 1u));
}

ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX unpack_clusterable_object_index_rangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(uint cluster_index) {
    uvec4 offset_and_counts_a = _group_0_binding_10_fs.data[cluster_index][0];
    uvec4 offset_and_counts_b = _group_0_binding_10_fs.data[cluster_index][1];
    uint point_light_offset = offset_and_counts_a.x;
    uint spot_light_offset = (point_light_offset + offset_and_counts_a.y);
    uint reflection_probe_offset = (spot_light_offset + offset_and_counts_a.z);
    uint irradiance_volume_offset = (reflection_probe_offset + offset_and_counts_a.w);
    uint decal_offset = (irradiance_volume_offset + offset_and_counts_b.x);
    uint last_clusterable_offset = (decal_offset + offset_and_counts_b.y);
    return ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(point_light_offset, spot_light_offset, reflection_probe_offset, irradiance_volume_offset, decal_offset, last_clusterable_offset);
}

uint get_clusterable_object_idX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(uint index) {
    uint _e4 = _group_0_binding_9_fs.data[index];
    return _e4;
}

vec4 cluster_debug_visualizationX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(vec4 input_color, float view_z_2, bool is_orthographic_2, ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX clusterable_object_index_ranges, uint cluster_index_1) {
    vec4 output_color = vec4(0.0);
    output_color = input_color;
    vec4 _e2 = output_color;
    return _e2;
}

float copysignX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(float a_1, float b) {
    return uintBitsToFloat(((floatBitsToUint(a_1) & 2147483647u) | (floatBitsToUint(b) & 2147483648u)));
}

mat3x3 orthonormalizeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(vec3 z_basis) {
    float _e3 = copysignX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(1.0, z_basis.z);
    float a_5 = (-1.0 / (_e3 + z_basis.z));
    float b_1 = ((z_basis.x * z_basis.y) * a_5);
    vec3 x_basis_2 = vec3((1.0 + (((_e3 * z_basis.x) * z_basis.x) * a_5)), (_e3 * b_1), (-(_e3) * z_basis.x));
    vec3 y_basis_2 = vec3(b_1, (_e3 + ((z_basis.y * z_basis.y) * a_5)), -(z_basis.y));
    return mat3x3(x_basis_2, y_basis_2, z_basis);
}

float interleaved_gradient_noiseX_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX(vec2 pixel_coordinates, uint frame) {
    vec2 xy_1 = (pixel_coordinates + vec2((5.588238 * float((frame % 64u)))));
    return fract((52.982918 * fract(((0.06711056 * xy_1.x) + (0.00583715 * xy_1.y)))));
}

float sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local, float depth, int array_index) {
    float _e5 = textureLod(_group_0_binding_5_fs, vec4(light_local, array_index, depth), 0.0);
    return _e5;
}

float sample_shadow_map_castano_thirteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_1, float depth_1, int array_index_1) {
    vec2 base_uv = vec2(0.0);
    float sum = 0.0;
    vec2 shadow_map_size = vec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    vec2 inv_shadow_map_size = (vec2(1.0) / shadow_map_size);
    vec2 uv_1 = (light_local_1 * shadow_map_size);
    base_uv = floor((uv_1 + vec2(0.5)));
    float _e18 = base_uv.x;
    float s = ((uv_1.x + 0.5) - _e18);
    float _e24 = base_uv.y;
    float t = ((uv_1.y + 0.5) - _e24);
    vec2 _e27 = base_uv;
    base_uv = (_e27 - vec2(0.5));
    vec2 _e30 = base_uv;
    base_uv = (_e30 * inv_shadow_map_size);
    float uw0_ = (4.0 - (3.0 * s));
    float uw2_ = (1.0 + (3.0 * s));
    float u0_ = (((3.0 - (2.0 * s)) / uw0_) - 2.0);
    float u1_ = ((3.0 + s) / 7.0);
    float u2_ = ((s / uw2_) + 2.0);
    float vw0_ = (4.0 - (3.0 * t));
    float vw2_ = (1.0 + (3.0 * t));
    float v0_ = (((3.0 - (2.0 * t)) / vw0_) - 2.0);
    float v1_ = ((3.0 + t) / 7.0);
    float v2_ = ((t / vw2_) + 2.0);
    vec2 _e77 = base_uv;
    float _e83 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e77 + (vec2(u0_, v0_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e86 = sum;
    sum = (_e86 + ((uw0_ * vw0_) * _e83));
    vec2 _e89 = base_uv;
    float _e93 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e89 + (vec2(u1_, v0_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e95 = sum;
    sum = (_e95 + ((7.0 * vw0_) * _e93));
    vec2 _e98 = base_uv;
    float _e102 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e98 + (vec2(u2_, v0_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e104 = sum;
    sum = (_e104 + ((uw2_ * vw0_) * _e102));
    vec2 _e107 = base_uv;
    float _e111 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e107 + (vec2(u0_, v1_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e113 = sum;
    sum = (_e113 + ((uw0_ * 7.0) * _e111));
    vec2 _e116 = base_uv;
    float _e120 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e116 + (vec2(u1_, v1_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e122 = sum;
    sum = (_e122 + ((7.0 * 7.0) * _e120));
    vec2 _e125 = base_uv;
    float _e129 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e125 + (vec2(u2_, v1_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e131 = sum;
    sum = (_e131 + ((uw2_ * 7.0) * _e129));
    vec2 _e134 = base_uv;
    float _e138 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e134 + (vec2(u0_, v2_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e140 = sum;
    sum = (_e140 + ((uw0_ * vw2_) * _e138));
    vec2 _e143 = base_uv;
    float _e147 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e143 + (vec2(u1_, v2_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e149 = sum;
    sum = (_e149 + ((7.0 * vw2_) * _e147));
    vec2 _e152 = base_uv;
    float _e156 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((_e152 + (vec2(u2_, v2_) * inv_shadow_map_size)), depth_1, array_index_1);
    float _e158 = sum;
    sum = (_e158 + ((uw2_ * vw2_) * _e156));
    float _e160 = sum;
    return (_e160 * 0.0069444445);
}

float sample_shadow_mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_2, float depth_2, int array_index_2, float texel_size) {
    float _e3 = sample_shadow_map_castano_thirteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_2, depth_2, array_index_2);
    return _e3;
}

vec2 search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_3, float depth_3, int array_index_3) {
    return vec2(0.0);
}

float search_for_blockers_in_shadow_mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_4, float depth_4, int array_index_4, float texel_size_1, float search_size) {
    vec2 sum_1 = vec2(0.0);
    vec2 shadow_map_size_1 = vec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    vec2 uv_offset_scale = (vec2(search_size) / (texel_size_1 * shadow_map_size_1));
    vec2 offset0_ = (vec2(0.125, -0.375) * uv_offset_scale);
    vec2 offset1_ = (vec2(-0.125, 0.375) * uv_offset_scale);
    vec2 offset2_ = (vec2(0.625, 0.125) * uv_offset_scale);
    vec2 offset3_ = (vec2(-0.375, -0.625) * uv_offset_scale);
    vec2 offset4_ = (vec2(-0.625, 0.625) * uv_offset_scale);
    vec2 offset5_ = (vec2(-0.875, -0.125) * uv_offset_scale);
    vec2 offset6_ = (vec2(0.375, 0.875) * uv_offset_scale);
    vec2 offset7_ = (vec2(0.875, -0.875) * uv_offset_scale);
    vec2 _e46 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset0_), depth_4, array_index_4);
    vec2 _e48 = sum_1;
    sum_1 = (_e48 + _e46);
    vec2 _e51 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset1_), depth_4, array_index_4);
    vec2 _e52 = sum_1;
    sum_1 = (_e52 + _e51);
    vec2 _e55 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset2_), depth_4, array_index_4);
    vec2 _e56 = sum_1;
    sum_1 = (_e56 + _e55);
    vec2 _e59 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset3_), depth_4, array_index_4);
    vec2 _e60 = sum_1;
    sum_1 = (_e60 + _e59);
    vec2 _e63 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset4_), depth_4, array_index_4);
    vec2 _e64 = sum_1;
    sum_1 = (_e64 + _e63);
    vec2 _e67 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset5_), depth_4, array_index_4);
    vec2 _e68 = sum_1;
    sum_1 = (_e68 + _e67);
    vec2 _e71 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset6_), depth_4, array_index_4);
    vec2 _e72 = sum_1;
    sum_1 = (_e72 + _e71);
    vec2 _e75 = search_for_blockers_in_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_4 + offset7_), depth_4, array_index_4);
    vec2 _e76 = sum_1;
    sum_1 = (_e76 + _e75);
    float _e79 = sum_1.y;
    if ((_e79 == 0.0)) {
        return 0.0;
    }
    float _e84 = sum_1.x;
    float _e86 = sum_1.y;
    return (_e84 / _e86);
}

mat2x2 random_rotation_matrixX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 scale, bool temporal) {
    uint _e4 = _group_0_binding_11_fs.frame_count;
    float _e7 = interleaved_gradient_noiseX_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX(scale, (temporal ? _e4 : 1u));
    float random_angle = (6.2831855 * _e7);
    vec2 m = vec2(sin(random_angle), cos(random_angle));
    return mat2x2(vec2(m.y, -(m.x)), vec2(m.x, m.y));
}

float mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(float min1_, float max1_, float min2_, float max2_, float value) {
    return (min2_ + (((value - min1_) * (max2_ - min2_)) / (max1_ - min1_)));
}

vec2 calculate_uv_offset_scale_jimenez_fourteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(float texel_size_2, float blur_size) {
    vec2 shadow_map_size_2 = vec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    float _e8 = mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(0.00390625, 0.022949219, 0.015, 0.035, texel_size_2);
    return (vec2((_e8 * blur_size)) / (texel_size_2 * shadow_map_size_2));
}

float sample_shadow_map_jimenez_fourteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_5, float depth_5, int array_index_5, float texel_size_3, float blur_size_1, bool temporal_1) {
    float sum_2 = 0.0;
    vec2 shadow_map_size_3 = vec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    mat2x2 _e7 = random_rotation_matrixX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 * shadow_map_size_3), temporal_1);
    vec2 _e10 = calculate_uv_offset_scale_jimenez_fourteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(texel_size_3, blur_size_1);
    vec2 sample_offset0_ = ((_e7 * SPIRAL_OFFSET_0_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset1_ = ((_e7 * SPIRAL_OFFSET_1_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset2_ = ((_e7 * SPIRAL_OFFSET_2_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset3_ = ((_e7 * SPIRAL_OFFSET_3_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset4_ = ((_e7 * SPIRAL_OFFSET_4_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset5_ = ((_e7 * SPIRAL_OFFSET_5_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset6_ = ((_e7 * SPIRAL_OFFSET_6_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    vec2 sample_offset7_ = ((_e7 * SPIRAL_OFFSET_7_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * _e10);
    float _e38 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset0_), depth_5, array_index_5);
    float _e40 = sum_2;
    sum_2 = (_e40 + _e38);
    float _e43 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset1_), depth_5, array_index_5);
    float _e44 = sum_2;
    sum_2 = (_e44 + _e43);
    float _e47 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset2_), depth_5, array_index_5);
    float _e48 = sum_2;
    sum_2 = (_e48 + _e47);
    float _e51 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset3_), depth_5, array_index_5);
    float _e52 = sum_2;
    sum_2 = (_e52 + _e51);
    float _e55 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset4_), depth_5, array_index_5);
    float _e56 = sum_2;
    sum_2 = (_e56 + _e55);
    float _e59 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset5_), depth_5, array_index_5);
    float _e60 = sum_2;
    sum_2 = (_e60 + _e59);
    float _e63 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset6_), depth_5, array_index_5);
    float _e64 = sum_2;
    sum_2 = (_e64 + _e63);
    float _e67 = sample_shadow_map_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((light_local_5 + sample_offset7_), depth_5, array_index_5);
    float _e68 = sum_2;
    sum_2 = (_e68 + _e67);
    float _e70 = sum_2;
    return (_e70 / 8.0);
}

float sample_shadow_map_pcssX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 light_local_6, float depth_6, int array_index_6, float texel_size_4, float light_size) {
    float _e5 = search_for_blockers_in_shadow_mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_6, depth_6, array_index_6, texel_size_4, light_size);
    float blur_size_2 = max((((_e5 - depth_6) * light_size) / depth_6), 0.5);
    float _e12 = sample_shadow_map_jimenez_fourteenX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_6, depth_6, array_index_6, texel_size_4, blur_size_2, false);
    return _e12;
}

float sample_shadow_cubemap_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_7, float depth_7, uint light_id_3) {
    float _e6 = textureLod(_group_0_binding_2_fs, vec4(light_local_7, int(light_id_3)), depth_7, 0.0);
    return _e6;
}

float sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 position, float coeff, vec3 x_basis, vec3 y_basis, vec3 light_local_8, float depth_8, uint light_id_4) {
    float _e12 = sample_shadow_cubemap_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(((light_local_8 + (position.x * x_basis)) + (position.y * y_basis)), depth_8, light_id_4);
    return (_e12 * coeff);
}

float sample_shadow_cubemap_gaussianX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_9, float depth_9, float scale_1, float distance_to_light, uint light_id_5) {
    float sum_3 = 0.0;
    mat3x3 _e3 = orthonormalizeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(normalize(light_local_9));
    mat3x3 basis = ((_e3 * scale_1) * distance_to_light);
    float _e16 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.125, -0.375), 0.157112, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e18 = sum_3;
    sum_3 = (_e18 + _e16);
    float _e26 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.125, 0.375), 0.157112, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e27 = sum_3;
    sum_3 = (_e27 + _e26);
    float _e35 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.625, 0.125), 0.138651, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e36 = sum_3;
    sum_3 = (_e36 + _e35);
    float _e44 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.375, -0.625), 0.130251, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e45 = sum_3;
    sum_3 = (_e45 + _e44);
    float _e53 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.625, 0.625), 0.114946, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e54 = sum_3;
    sum_3 = (_e54 + _e53);
    float _e62 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.875, -0.125), 0.114946, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e63 = sum_3;
    sum_3 = (_e63 + _e62);
    float _e71 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.375, 0.875), 0.107982, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e72 = sum_3;
    sum_3 = (_e72 + _e71);
    float _e80 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.875, -0.875), 0.079001, basis[0], basis[1], light_local_9, depth_9, light_id_5);
    float _e81 = sum_3;
    sum_3 = (_e81 + _e80);
    float _e83 = sum_3;
    return _e83;
}

float sample_shadow_cubemapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_10, float distance_to_light_1, float depth_10, uint light_id_6) {
    float _e5 = sample_shadow_cubemap_gaussianX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_10, depth_10, POINT_SHADOW_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX, distance_to_light_1, light_id_6);
    return _e5;
}

vec2 search_for_blockers_in_shadow_cubemap_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_11, float depth_11, uint light_id_7) {
    return vec2(0.0);
}

vec2 search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2 position_1, vec3 x_basis_1, vec3 y_basis_1, vec3 light_local_12, float depth_12, uint light_id_8) {
    vec2 _e12 = search_for_blockers_in_shadow_cubemap_hardwareX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(((light_local_12 + (position_1.x * x_basis_1)) + (position_1.y * y_basis_1)), depth_12, light_id_8);
    return _e12;
}

float search_for_blockers_in_shadow_cubemapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_13, float depth_13, float scale_2, float distance_to_light_2, uint light_id_9) {
    vec2 sum_4 = vec2(0.0);
    mat3x3 _e4 = orthonormalizeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(normalize(light_local_13));
    mat3x3 basis_1 = ((_e4 * scale_2) * distance_to_light_2);
    vec2 _e16 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.125, -0.375), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e18 = sum_4;
    sum_4 = (_e18 + _e16);
    vec2 _e25 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.125, 0.375), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e26 = sum_4;
    sum_4 = (_e26 + _e25);
    vec2 _e33 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.625, 0.125), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e34 = sum_4;
    sum_4 = (_e34 + _e33);
    vec2 _e41 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.375, -0.625), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e42 = sum_4;
    sum_4 = (_e42 + _e41);
    vec2 _e49 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.625, 0.625), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e50 = sum_4;
    sum_4 = (_e50 + _e49);
    vec2 _e57 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(-0.875, -0.125), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e58 = sum_4;
    sum_4 = (_e58 + _e57);
    vec2 _e65 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.375, 0.875), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e66 = sum_4;
    sum_4 = (_e66 + _e65);
    vec2 _e73 = search_for_blockers_in_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(0.875, -0.875), basis_1[0], basis_1[1], light_local_13, depth_13, light_id_9);
    vec2 _e74 = sum_4;
    sum_4 = (_e74 + _e73);
    float _e77 = sum_4.y;
    if ((_e77 == 0.0)) {
        return 0.0;
    }
    float _e82 = sum_4.x;
    float _e84 = sum_4.y;
    return (_e82 / _e84);
}

float sample_shadow_cubemap_jitteredX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_14, float depth_14, float scale_3, float distance_to_light_3, uint light_id_10, bool temporal_2) {
    float sum_5 = 0.0;
    mat3x3 _e3 = orthonormalizeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(normalize(light_local_14));
    mat3x3 basis_2 = ((_e3 * scale_3) * distance_to_light_3);
    mat2x2 _e11 = random_rotation_matrixX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec2(1.0), temporal_2);
    vec2 sample_offset0_1 = ((_e11 * SPIRAL_OFFSET_0_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset1_1 = ((_e11 * SPIRAL_OFFSET_1_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset2_1 = ((_e11 * SPIRAL_OFFSET_2_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset3_1 = ((_e11 * SPIRAL_OFFSET_3_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset4_1 = ((_e11 * SPIRAL_OFFSET_4_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset5_1 = ((_e11 * SPIRAL_OFFSET_5_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset6_1 = ((_e11 * SPIRAL_OFFSET_6_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    vec2 sample_offset7_1 = ((_e11 * SPIRAL_OFFSET_7_X_naga_oil_mod_XMJSXM6K7OBRHEOR2OV2GS3DTX) * POINT_SHADOW_TEMPORAL_OFFSET_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    float _e49 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset0_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e51 = sum_5;
    sum_5 = (_e51 + _e49);
    float _e56 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset1_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e57 = sum_5;
    sum_5 = (_e57 + _e56);
    float _e62 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset2_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e63 = sum_5;
    sum_5 = (_e63 + _e62);
    float _e68 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset3_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e69 = sum_5;
    sum_5 = (_e69 + _e68);
    float _e74 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset4_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e75 = sum_5;
    sum_5 = (_e75 + _e74);
    float _e80 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset5_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e81 = sum_5;
    sum_5 = (_e81 + _e80);
    float _e86 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset6_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e87 = sum_5;
    sum_5 = (_e87 + _e86);
    float _e92 = sample_shadow_cubemap_at_offsetX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(sample_offset7_1, 0.125, basis_2[0], basis_2[1], light_local_14, depth_14, light_id_10);
    float _e93 = sum_5;
    sum_5 = (_e93 + _e92);
    float _e95 = sum_5;
    return _e95;
}

float sample_shadow_cubemap_pcssX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(vec3 light_local_15, float distance_to_light_4, float depth_15, uint light_id_11, float light_size_1) {
    float _e5 = search_for_blockers_in_shadow_cubemapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_15, depth_15, light_size_1, distance_to_light_4, light_id_11);
    float blur_size_3 = max((((_e5 - depth_15) * light_size_1) / depth_15), 0.5);
    float _e14 = sample_shadow_cubemap_jitteredX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(light_local_15, depth_15, (POINT_SHADOW_SCALEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX * blur_size_3), distance_to_light_4, light_id_11, false);
    return _e14;
}

vec3 hsv_to_rgbX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DUY3PNRXXEX3POBSXEYLUNFXW44YX(vec3 hsv) {
    vec3 n = vec3(5.0, 3.0, 1.0);
    vec3 k_1 = ((n + vec3((hsv.x / FRAC_PI_3X_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX))) - vec3(6.0) * trunc((n + vec3((hsv.x / FRAC_PI_3X_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX))) / vec3(6.0)));
    return (vec3(hsv.z) - ((hsv.z * hsv.y) * max(vec3(0.0), min(k_1, min((vec3(4.0) - k_1), vec3(1.0))))));
}

float fetch_point_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_12, vec4 frag_position, vec3 surface_normal) {
    vec4 _e6 = _group_0_binding_8_fs.data[light_id_12].position_radius;
    vec3 surface_to_light = (_e6.xyz - frag_position.xyz);
    vec3 surface_to_light_abs = abs(surface_to_light);
    float distance_to_light_5 = max(surface_to_light_abs.x, max(surface_to_light_abs.y, surface_to_light_abs.z));
    float _e18 = _group_0_binding_8_fs.data[light_id_12].shadow_normal_bias;
    vec3 normal_offset = ((_e18 * distance_to_light_5) * surface_normal.xyz);
    float _e23 = _group_0_binding_8_fs.data[light_id_12].shadow_depth_bias;
    vec3 depth_offset = (_e23 * normalize(surface_to_light.xyz));
    vec3 offset_position_1 = ((frag_position.xyz + normal_offset) + depth_offset);
    vec4 _e32 = _group_0_binding_8_fs.data[light_id_12].position_radius;
    vec3 frag_ls = (offset_position_1.xyz - _e32.xyz);
    vec3 abs_position_ls = abs(frag_ls);
    float major_axis_magnitude = max(abs_position_ls.x, max(abs_position_ls.y, abs_position_ls.z));
    vec4 _e43 = _group_0_binding_8_fs.data[light_id_12].light_custom_data;
    vec4 _e47 = _group_0_binding_8_fs.data[light_id_12].light_custom_data;
    vec2 zw = ((-(major_axis_magnitude) * _e43.xy) + _e47.zw);
    float depth_16 = (zw.x / zw.y);
    float _e54 = _group_0_binding_8_fs.data[light_id_12].soft_shadow_size;
    if ((_e54 > 0.0)) {
        float _e60 = _group_0_binding_8_fs.data[light_id_12].soft_shadow_size;
        float _e61 = sample_shadow_cubemap_pcssX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((frag_ls * flip_zX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX), distance_to_light_5, depth_16, light_id_12, _e60);
        return _e61;
    }
    float _e64 = sample_shadow_cubemapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX((frag_ls * flip_zX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX), distance_to_light_5, depth_16, light_id_12);
    return _e64;
}

float fetch_spot_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_13, vec4 frag_position_1, vec3 surface_normal_1, float near_z) {
    vec3 spot_dir_1 = vec3(0.0);
    vec4 _e6 = _group_0_binding_8_fs.data[light_id_13].position_radius;
    vec3 surface_to_light_1 = (_e6.xyz - frag_position_1.xyz);
    float _e12 = _group_0_binding_8_fs.data[light_id_13].light_custom_data.x;
    float _e15 = _group_0_binding_8_fs.data[light_id_13].light_custom_data.y;
    spot_dir_1 = vec3(_e12, 0.0, _e15);
    float _e21 = spot_dir_1.x;
    float _e23 = spot_dir_1.x;
    float _e28 = spot_dir_1.z;
    float _e30 = spot_dir_1.z;
    spot_dir_1.y = sqrt(max(0.0, ((1.0 - (_e21 * _e23)) - (_e28 * _e30))));
    uint _e37 = _group_0_binding_8_fs.data[light_id_13].flags;
    if (((_e37 & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVEX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX) != 0u)) {
        float _e44 = spot_dir_1.y;
        spot_dir_1.y = -(_e44);
    }
    vec3 _e46 = spot_dir_1;
    vec3 fwd = -(_e46);
    float distance_to_light_6 = dot(fwd, surface_to_light_1);
    float _e52 = _group_0_binding_8_fs.data[light_id_13].shadow_depth_bias;
    float _e58 = _group_0_binding_8_fs.data[light_id_13].shadow_normal_bias;
    vec3 offset_position_2 = ((-(surface_to_light_1) + (_e52 * normalize(surface_to_light_1))) + ((surface_normal_1.xyz * _e58) * distance_to_light_6));
    mat3x3 _e62 = orthonormalizeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(fwd);
    vec3 projected_position = (offset_position_2 * _e62);
    float _e65 = _group_0_binding_8_fs.data[light_id_13].spot_light_tan_angle;
    float f_div_minus_z = (1.0 / (_e65 * -(projected_position.z)));
    vec2 shadow_xy_ndc = (projected_position.xy * f_div_minus_z);
    vec2 shadow_uv = ((shadow_xy_ndc * vec2(0.5, -0.5)) + vec2(0.5, 0.5));
    float depth_17 = (near_z / -(projected_position.z));
    int _e88 = _group_0_binding_1_fs.spot_light_shadowmap_offset;
    int array_index_7 = (int(light_id_13) + _e88);
    float _e91 = _group_0_binding_8_fs.data[light_id_13].soft_shadow_size;
    if ((_e91 > 0.0)) {
        float _e95 = _group_0_binding_8_fs.data[light_id_13].soft_shadow_size;
        float _e97 = sample_shadow_map_pcssX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(shadow_uv, depth_17, array_index_7, SPOT_SHADOW_TEXEL_SIZEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX, _e95);
        return _e97;
    }
    float _e99 = sample_shadow_mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(shadow_uv, depth_17, array_index_7, SPOT_SHADOW_TEXEL_SIZEX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX);
    return _e99;
}

uint get_cascade_indexX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_14, float view_z_3) {
    uint i = 0u;
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e19 = i;
            i = (_e19 + 1u);
        }
        loop_init = false;
        uint _e6 = i;
        uint _e8 = _group_0_binding_1_fs.directional_lights[light_id_14].num_cascades;
        if ((_e6 < _e8)) {
        } else {
            break;
        }
        {
            uint _e13 = i;
            float _e16 = _group_0_binding_1_fs.directional_lights[light_id_14].cascades[_e13].far_bound;
            if ((-(view_z_3) < _e16)) {
                uint _e18 = i;
                return _e18;
            }
        }
    }
    uint _e23 = _group_0_binding_1_fs.directional_lights[light_id_14].num_cascades;
    return _e23;
}

vec4 world_to_directional_light_localX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_15, uint cascade_index, vec4 offset_position) {
    mat4x4 _e9 = _group_0_binding_1_fs.directional_lights[light_id_15].cascades[cascade_index].clip_from_world;
    vec4 offset_position_clip = (_e9 * offset_position);
    if ((offset_position_clip.w <= 0.0)) {
        return vec4(0.0);
    }
    vec3 offset_position_ndc = (offset_position_clip.xyz / vec3(offset_position_clip.w));
    if (((any(lessThan(offset_position_ndc.xy, vec2(-1.0))) || (offset_position_ndc.z < 0.0)) || any(greaterThan(offset_position_ndc, vec3(1.0))))) {
        return vec4(0.0);
    }
    vec2 flip_correction = vec2(0.5, -0.5);
    vec2 light_local_16 = ((offset_position_ndc.xy * flip_correction) + vec2(0.5, 0.5));
    float depth_18 = offset_position_ndc.z;
    return vec4(light_local_16, depth_18, 1.0);
}

float sample_directional_cascadeX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_16, uint cascade_index_1, vec4 frag_position_2, vec3 surface_normal_2) {
    float _e9 = _group_0_binding_1_fs.directional_lights[light_id_16].shadow_normal_bias;
    float _e11 = _group_0_binding_1_fs.directional_lights[light_id_16].cascades[cascade_index_1].texel_size;
    vec3 normal_offset_1 = ((_e9 * _e11) * surface_normal_2.xyz);
    float _e16 = _group_0_binding_1_fs.directional_lights[light_id_16].shadow_depth_bias;
    vec3 _e18 = _group_0_binding_1_fs.directional_lights[light_id_16].direction_to_light;
    vec3 depth_offset_1 = (_e16 * _e18.xyz);
    vec4 offset_position_3 = vec4(((frag_position_2.xyz + normal_offset_1) + depth_offset_1), frag_position_2.w);
    vec4 _e27 = world_to_directional_light_localX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(light_id_16, cascade_index_1, offset_position_3);
    if ((_e27.w == 0.0)) {
        return 1.0;
    }
    uint _e33 = _group_0_binding_1_fs.directional_lights[light_id_16].depth_texture_base_index;
    int array_index_8 = int((_e33 + cascade_index_1));
    float texel_size_5 = _group_0_binding_1_fs.directional_lights[light_id_16].cascades[cascade_index_1].texel_size;
    float _e39 = _group_0_binding_1_fs.directional_lights[light_id_16].soft_shadow_size;
    if ((_e39 > 0.0)) {
        float _e45 = _group_0_binding_1_fs.directional_lights[light_id_16].soft_shadow_size;
        float _e46 = sample_shadow_map_pcssX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(_e27.xy, _e27.z, array_index_8, texel_size_5, _e45);
        return _e46;
    }
    float _e49 = sample_shadow_mapX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5PXGYLNOBWGS3THX(_e27.xy, _e27.z, array_index_8, texel_size_5);
    return _e49;
}

float fetch_directional_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(uint light_id_17, vec4 frag_position_3, vec3 surface_normal_3, float view_z_4) {
    float shadow = 0.0;
    uint _e5 = get_cascade_indexX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(light_id_17, view_z_4);
    uint _e7 = _group_0_binding_1_fs.directional_lights[light_id_17].num_cascades;
    if ((_e5 >= _e7)) {
        return 1.0;
    }
    float _e12 = sample_directional_cascadeX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(light_id_17, _e5, frag_position_3, surface_normal_3);
    shadow = _e12;
    uint next_cascade_index = (_e5 + 1u);
    uint _e17 = _group_0_binding_1_fs.directional_lights[light_id_17].num_cascades;
    if ((next_cascade_index < _e17)) {
        float this_far_bound = _group_0_binding_1_fs.directional_lights[light_id_17].cascades[_e5].far_bound;
        float _e24 = _group_0_binding_1_fs.directional_lights[light_id_17].cascades_overlap_proportion;
        float next_near_bound = ((1.0 - _e24) * this_far_bound);
        if ((-(view_z_4) >= next_near_bound)) {
            float _e30 = sample_directional_cascadeX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(light_id_17, next_cascade_index, frag_position_3, surface_normal_3);
            float _e31 = shadow;
            shadow = mix(_e31, _e30, ((-(view_z_4) - next_near_bound) / (this_far_bound - next_near_bound)));
        }
    }
    float _e37 = shadow;
    return _e37;
}

vec3 EnvBRDFApproxX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3 F0_1, vec2 F_ab_1) {
    return ((F0_1 * F_ab_1.x) + vec3(F_ab_1.y));
}

vec3 ambient_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2MFWWE2LFNZ2AX(vec4 world_position, vec3 world_normal, vec3 V_3, float NdotV_2, vec3 diffuse_color, vec3 specular_color, float perceptual_roughness_1, vec3 occlusion) {
    vec2 _e2 = F_ABX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(1.0, NdotV_2);
    vec3 _e4 = EnvBRDFApproxX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(diffuse_color, _e2);
    vec2 _e6 = F_ABX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(perceptual_roughness_1, NdotV_2);
    vec3 _e8 = EnvBRDFApproxX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(specular_color, _e6);
    float specular_occlusion_1 = clamp(dot(specular_color, vec3(16.5)), 0.0, 1.0);
    vec4 _e18 = _group_0_binding_1_fs.ambient_color;
    return (((_e4 + (_e8 * specular_occlusion_1)) * _e18.xyz) * occlusion);
}

vec3 powsafeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(vec3 color_1, float power) {
    return (pow(abs(color_1), vec3(power)) * sign(color_1));
}

vec3 screen_space_ditherX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(vec2 frag_coord_1) {
    vec3 dither = vec3(0.0);
    dither = vec3(dot(vec2(171.0, 231.0), frag_coord_1)).xxx;
    vec3 _e8 = dither;
    dither = fract((_e8.xyz / vec3(103.0, 71.0, 97.0)));
    vec3 _e16 = dither;
    return ((_e16 - vec3(0.5)) / vec3(255.0));
}

float tonemapping_luminanceX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(vec3 v) {
    return dot(v, vec3(0.2126, 0.7152, 0.0722));
}

vec3 saturationX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(vec3 color_2, float saturationAmount) {
    float _e1 = tonemapping_luminanceX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(color_2);
    return mix(vec3(_e1), color_2, vec3(saturationAmount));
}

vec4 tone_mappingX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(vec4 in_1, ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX in_color_grading) {
    vec3 color_3 = vec3(0.0);
    ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX color_grading = ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX(mat3x3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec2(0.0), 0.0, 0.0, 0.0);
    color_3 = max(in_1.xyz, vec3(0.0));
    color_grading = in_color_grading;
    vec3 _e8 = color_3;
    float _e12 = color_grading.exposure;
    vec3 _e13 = powsafeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(vec3(2.0), _e12);
    color_3 = (_e8 * _e13);
    vec3 _e15 = color_3;
    color_3 = _e15;
    vec3 _e16 = color_3;
    float _e18 = color_grading.post_saturation;
    vec3 _e19 = saturationX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(_e16, _e18);
    color_3 = _e19;
    vec3 _e20 = color_3;
    return vec4(_e20, in_1.w);
}

vec3 prepare_world_normalX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(vec3 world_normal_1, bool double_sided, bool is_front_1) {
    vec3 output_ = vec3(0.0);
    output_ = world_normal_1;
    vec3 _e11 = output_;
    output_ = (((float((!(double_sided) || is_front_1)) * 2.0) - 1.0) * _e11);
    vec3 _e13 = output_;
    return _e13;
}

vec3 calculate_viewX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(vec4 world_position_1, bool is_orthographic_3) {
    vec3 V_4 = vec3(0.0);
    if (is_orthographic_3) {
        float _e5 = _group_0_binding_0_fs.clip_from_world[0][2];
        float _e10 = _group_0_binding_0_fs.clip_from_world[1][2];
        float _e15 = _group_0_binding_0_fs.clip_from_world[2][2];
        V_4 = normalize(vec3(_e5, _e10, _e15));
    } else {
        vec3 _e22 = _group_0_binding_0_fs.world_position;
        V_4 = normalize((_e22.xyz - world_position_1.xyz));
    }
    vec3 _e27 = V_4;
    return _e27;
}

PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_from_vertex_outputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOJQWO3LFNZ2AX(VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX in_2, bool is_front_2, bool double_sided_1) {
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_2 = PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(vec4(0.0), vec4(0.0), vec4(0.0), mat3x3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, vec2(0.0), 0u, 0.0, 0.0, 0.0, 0.0, 0u, 0u), vec3(0.0), 0.0, vec4(0.0), vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), 0.0, vec3(0.0), vec3(0.0), false, 0u);
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e0 = pbr_input_newX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX();
    pbr_input_2 = _e0;
    uint _e8 = _group_2_binding_0_fs[in_2.instance_index].flags;
    pbr_input_2.flags = _e8;
    float _e14 = _group_0_binding_0_fs.clip_from_view[3][3];
    pbr_input_2.is_orthographic = (_e14 == 1.0);
    bool _e20 = pbr_input_2.is_orthographic;
    vec3 _e21 = calculate_viewX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(in_2.world_position, _e20);
    pbr_input_2.V = _e21;
    pbr_input_2.frag_coord = in_2.position;
    pbr_input_2.world_position = in_2.world_position;
    vec3 _e30 = prepare_world_normalX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(in_2.world_normal, double_sided_1, is_front_2);
    pbr_input_2.world_normal = _e30;
    vec3 _e33 = pbr_input_2.world_normal;
    pbr_input_2.N = normalize(_e33);
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e35 = pbr_input_2;
    return _e35;
}

PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_from_standard_materialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOJQWO3LFNZ2AX(VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX in_3, bool is_front_3) {
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_3 = PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(vec4(0.0), vec4(0.0), vec4(0.0), mat3x3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, vec2(0.0), 0u, 0.0, 0.0, 0.0, 0.0, 0u, 0u), vec3(0.0), 0.0, vec4(0.0), vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), 0.0, vec3(0.0), vec3(0.0), false, 0u);
    SampleBiasX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX bias = SampleBiasX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(0.0);
    vec2 uv = vec2(0.0);
    vec2 uv_b = vec2(0.0);
    vec4 emissive = vec4(0.0);
    float metallic = 0.0;
    float perceptual_roughness_2 = 0.0;
    float specular_transmission = 0.0;
    float thickness = 0.0;
    float diffuse_transmission = 0.0;
    vec3 diffuse_occlusion = vec3(1.0);
    float specular_occlusion = 1.0;
    uint _e7 = _group_2_binding_0_fs[in_3.instance_index].material_and_lightmap_bind_group_slot;
    uint slot = (_e7 & 65535u);
    uint flags = _group_3_binding_0_fs.flags;
    vec4 base_color_3 = _group_3_binding_0_fs.base_color;
    uint deferred_lighting_pass_id = _group_3_binding_0_fs.deferred_lighting_pass_id;
    bool double_sided_2 = ((flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) != 0u);
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e24 = pbr_input_from_vertex_outputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOJQWO3LFNZ2AX(in_3, is_front_3, double_sided_2);
    pbr_input_3 = _e24;
    pbr_input_3.material.flags = flags;
    vec4 _e30 = pbr_input_3.material.base_color;
    pbr_input_3.material.base_color = (_e30 * base_color_3);
    pbr_input_3.material.deferred_lighting_pass_id = deferred_lighting_pass_id;
    vec3 _e35 = pbr_input_3.N;
    vec3 _e37 = pbr_input_3.V;
    float NdotV_6 = max(dot(_e35, _e37), 0.0001);
    float _e45 = _group_0_binding_0_fs.mip_bias;
    bias.mip_bias = _e45;
    mat3x3 uv_transform = _group_3_binding_0_fs.uv_transform;
    pbr_input_3.material.uv_transform = uv_transform;
    uv = (uv_transform * vec3(in_3.uv, 1.0)).xy;
    vec2 _e57 = uv;
    uv_b = _e57;
    if (((flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) != 0u)) {
        vec2 _e67 = uv;
        float _e69 = bias.mip_bias;
        vec4 _e70 = texture(_group_3_binding_1_fs, vec2(_e67), _e69);
        vec4 _e71 = pbr_input_3.material.base_color;
        pbr_input_3.material.base_color = (_e71 * _e70);
    }
    pbr_input_3.material.flags = flags;
    if (((flags & STANDARD_MATERIAL_FLAGS_UNLIT_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) == 0u)) {
        float _e83 = _group_3_binding_0_fs.ior;
        pbr_input_3.material.ior = _e83;
        vec4 _e88 = _group_3_binding_0_fs.attenuation_color;
        pbr_input_3.material.attenuation_color = _e88;
        float _e93 = _group_3_binding_0_fs.attenuation_distance;
        pbr_input_3.material.attenuation_distance = _e93;
        float _e98 = _group_3_binding_0_fs.alpha_cutoff;
        pbr_input_3.material.alpha_cutoff = _e98;
        vec3 _e103 = _group_3_binding_0_fs.reflectance;
        pbr_input_3.material.reflectance = _e103;
        vec4 _e106 = _group_3_binding_0_fs.emissive;
        emissive = _e106;
        if (((flags & STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) != 0u)) {
            vec4 _e112 = emissive;
            vec2 _e116 = uv;
            float _e118 = bias.mip_bias;
            vec4 _e119 = texture(_group_3_binding_3_fs, vec2(_e116), _e118);
            float _e123 = emissive.w;
            emissive = vec4((_e112.xyz * _e119.xyz), _e123);
        }
        vec4 _e127 = emissive;
        pbr_input_3.material.emissive = _e127;
        float _e130 = _group_3_binding_0_fs.metallic;
        metallic = _e130;
        float _e134 = _group_3_binding_0_fs.perceptual_roughness;
        perceptual_roughness_2 = _e134;
        if (((flags & STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) != 0u)) {
            vec2 _e142 = uv;
            float _e144 = bias.mip_bias;
            vec4 metallic_roughness = texture(_group_3_binding_5_fs, vec2(_e142), _e144);
            float _e147 = metallic;
            metallic = (_e147 * metallic_roughness.z);
            float _e150 = perceptual_roughness_2;
            perceptual_roughness_2 = (_e150 * metallic_roughness.y);
        }
        float _e154 = metallic;
        pbr_input_3.material.metallic = _e154;
        float _e157 = perceptual_roughness_2;
        pbr_input_3.material.perceptual_roughness = _e157;
        float _e162 = _group_3_binding_0_fs.clearcoat;
        pbr_input_3.material.clearcoat = _e162;
        float _e167 = _group_3_binding_0_fs.clearcoat_perceptual_roughness;
        pbr_input_3.material.clearcoat_perceptual_roughness = _e167;
        float _e170 = _group_3_binding_0_fs.specular_transmission;
        specular_transmission = _e170;
        float _e174 = specular_transmission;
        pbr_input_3.material.specular_transmission = _e174;
        float _e177 = _group_3_binding_0_fs.thickness;
        thickness = _e177;
        mat3x4 _e183 = _group_2_binding_0_fs[in_3.instance_index].world_from_local;
        vec3 _e186 = pbr_input_3.N;
        float _e192 = thickness;
        thickness = (_e192 * length((transpose(_e183) * vec4(_e186, 0.0)).xyz));
        float _e196 = thickness;
        pbr_input_3.material.thickness = _e196;
        float _e199 = _group_3_binding_0_fs.diffuse_transmission;
        diffuse_transmission = _e199;
        float _e203 = diffuse_transmission;
        pbr_input_3.material.diffuse_transmission = _e203;
        if (((flags & STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) != 0u)) {
            vec2 _e211 = uv;
            float _e213 = bias.mip_bias;
            vec4 _e214 = texture(_group_3_binding_7_fs, vec2(_e211), _e213);
            vec3 _e216 = diffuse_occlusion;
            diffuse_occlusion = (_e216 * _e214.x);
        }
        vec3 _e219 = diffuse_occlusion;
        pbr_input_3.diffuse_occlusion = _e219;
        float _e222 = specular_occlusion;
        pbr_input_3.specular_occlusion = _e222;
        vec3 _e225 = pbr_input_3.world_normal;
        pbr_input_3.N = normalize(_e225);
        vec3 _e229 = pbr_input_3.N;
        pbr_input_3.clearcoat_N = _e229;
    }
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e230 = pbr_input_3;
    return _e230;
}

vec4 alpha_discardX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX material_1, vec4 output_color_1) {
    vec4 color_4 = vec4(0.0);
    color_4 = output_color_1;
    uint alpha_mode = (material_1.flags & STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITSX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX);
    if ((alpha_mode == STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUEX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX)) {
        color_4.w = 1.0;
    }
    vec4 _e10 = color_4;
    return _e10;
}

vec3 calculate_diffuse_colorX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(vec3 base_color, float metallic_1, float specular_transmission_1, float diffuse_transmission_1) {
    return (((base_color * (1.0 - metallic_1)) * (1.0 - specular_transmission_1)) * (1.0 - diffuse_transmission_1));
}

vec3 calculate_F0X_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(vec3 base_color_1, float metallic_2, vec3 reflectance) {
    return ((((0.16 * reflectance) * reflectance) * (1.0 - metallic_2)) + (base_color_1 * metallic_2));
}

vec4 apply_pbr_lightingX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX in_4) {
    vec4 output_color_2 = vec4(0.0);
    vec3 direct_light = vec3(0.0);
    vec3 transmitted_light = vec3(0.0);
    LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX lighting_input = LightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(LayerLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX[1](LayerLightingInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(vec3(0.0), vec3(0.0), 0.0, 0.0, 0.0)), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec2(0.0));
    ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX clusterable_object_index_ranges_1 = ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(0u, 0u, 0u, 0u, 0u, 0u);
    uint i_1 = 0u;
    float shadow_1 = 0.0;
    uint i_2 = 0u;
    float shadow_2 = 0.0;
    uint i_3 = 0u;
    float shadow_3 = 0.0;
    vec3 light_contrib = vec3(0.0);
    vec3 indirect_light = vec3(0.0);
    bool found_diffuse_indirect = false;
    vec3 specular_transmitted_environment_light = vec3(0.0);
    vec3 emissive_light = vec3(0.0);
    output_color_2 = in_4.material.base_color;
    vec4 emissive_2 = in_4.material.emissive;
    float metallic_4 = in_4.material.metallic;
    float perceptual_roughness_4 = in_4.material.perceptual_roughness;
    float _e14 = perceptualRoughnessToRoughnessX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(perceptual_roughness_4);
    float ior = in_4.material.ior;
    float thickness_1 = in_4.material.thickness;
    vec3 reflectance_1 = in_4.material.reflectance;
    float diffuse_transmission_2 = in_4.material.diffuse_transmission;
    float specular_transmission_2 = in_4.material.specular_transmission;
    vec3 specular_transmissive_color = (specular_transmission_2 * in_4.material.base_color.xyz);
    vec3 diffuse_occlusion_1 = in_4.diffuse_occlusion;
    float specular_occlusion_2 = in_4.specular_occlusion;
    float NdotV_7 = max(dot(in_4.N, in_4.V), 0.0001);
    vec3 R_1 = reflect(-(in_4.V), in_4.N);
    vec4 _e40 = output_color_2;
    vec3 _e42 = calculate_diffuse_colorX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(_e40.xyz, metallic_4, specular_transmission_2, diffuse_transmission_2);
    vec4 _e43 = output_color_2;
    vec3 diffuse_transmissive_color = (((_e43.xyz * (1.0 - metallic_4)) * (1.0 - specular_transmission_2)) * diffuse_transmission_2);
    vec4 diffuse_transmissive_lobe_world_position = (in_4.world_position - (vec4(in_4.world_normal, 0.0) * thickness_1));
    vec4 _e58 = output_color_2;
    vec3 _e60 = calculate_F0X_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(_e58.xyz, metallic_4, reflectance_1);
    vec2 _e61 = F_ABX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(perceptual_roughness_4, NdotV_7);
    lighting_input.layers[0].NdotV = NdotV_7;
    lighting_input.layers[0].N = in_4.N;
    lighting_input.layers[0].R = R_1;
    lighting_input.layers[0].perceptual_roughness = perceptual_roughness_4;
    lighting_input.layers[0].roughness = _e14;
    lighting_input.P = in_4.world_position.xyz;
    lighting_input.V = in_4.V;
    lighting_input.diffuse_color = _e42;
    lighting_input.F0_ = _e60;
    lighting_input.F_ab = _e61;
    float _e91 = _group_0_binding_0_fs.view_from_world[0][2];
    float _e96 = _group_0_binding_0_fs.view_from_world[1][2];
    float _e101 = _group_0_binding_0_fs.view_from_world[2][2];
    float _e106 = _group_0_binding_0_fs.view_from_world[3][2];
    float view_z_5 = dot(vec4(_e91, _e96, _e101, _e106), in_4.world_position);
    uint _e113 = fragment_cluster_indexX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(in_4.frag_coord.xy, view_z_5, in_4.is_orthographic);
    ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX _e114 = unpack_clusterable_object_index_rangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(_e113);
    clusterable_object_index_ranges_1 = _e114;
    uint _e117 = clusterable_object_index_ranges_1.first_point_light_index_offset;
    i_1 = _e117;
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            uint _e153 = i_1;
            i_1 = (_e153 + 1u);
        }
        loop_init_1 = false;
        uint _e119 = i_1;
        uint _e121 = clusterable_object_index_ranges_1.first_spot_light_index_offset;
        if ((_e119 < _e121)) {
        } else {
            break;
        }
        {
            uint _e123 = i_1;
            uint _e124 = get_clusterable_object_idX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(_e123);
            shadow_1 = 1.0;
            uint _e136 = _group_0_binding_8_fs.data[_e124].flags;
            if ((((in_4.flags & MESH_FLAGS_SHADOW_RECEIVER_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX) != 0u) && ((_e136 & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX) != 0u))) {
                float _e144 = fetch_point_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(_e124, in_4.world_position, in_4.world_normal);
                shadow_1 = _e144;
            }
            vec3 _e147 = point_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(_e124, lighting_input, true, true);
            float _e149 = shadow_1;
            vec3 _e151 = direct_light;
            direct_light = (_e151 + (_e147 * _e149));
        }
    }
    uint _e157 = clusterable_object_index_ranges_1.first_spot_light_index_offset;
    i_2 = _e157;
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            uint _e196 = i_2;
            i_2 = (_e196 + 1u);
        }
        loop_init_2 = false;
        uint _e159 = i_2;
        uint _e161 = clusterable_object_index_ranges_1.first_reflection_probe_index_offset;
        if ((_e159 < _e161)) {
        } else {
            break;
        }
        {
            uint _e163 = i_2;
            uint _e164 = get_clusterable_object_idX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(_e163);
            shadow_2 = 1.0;
            uint _e176 = _group_0_binding_8_fs.data[_e164].flags;
            if ((((in_4.flags & MESH_FLAGS_SHADOW_RECEIVER_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX) != 0u) && ((_e176 & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX) != 0u))) {
                float _e188 = _group_0_binding_8_fs.data[_e164].shadow_map_near_z;
                float _e189 = fetch_spot_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(_e164, in_4.world_position, in_4.world_normal, _e188);
                shadow_2 = _e189;
            }
            vec3 _e191 = spot_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(_e164, lighting_input, true);
            float _e192 = shadow_2;
            vec3 _e194 = direct_light;
            direct_light = (_e194 + (_e191 * _e192));
        }
    }
    uint n_directional_lights = _group_0_binding_1_fs.n_directional_lights;
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            uint _e240 = i_3;
            i_3 = (_e240 + 1u);
        }
        loop_init_3 = false;
        uint _e203 = i_3;
        if ((_e203 < n_directional_lights)) {
        } else {
            break;
        }
        {
            uint _e207 = i_3;
            shadow_3 = 1.0;
            uint _e218 = i_3;
            uint _e221 = _group_0_binding_1_fs.directional_lights[_e218].flags;
            if ((((in_4.flags & MESH_FLAGS_SHADOW_RECEIVER_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OR4XAZLTX) != 0u) && ((_e221 & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2NVSXG2C7OZUWK527OR4XAZLTX) != 0u))) {
                uint _e227 = i_3;
                float _e230 = fetch_directional_shadowX_naga_oil_mod_XMJSXM6K7OBRHEOR2ONUGCZDPO5ZQX(_e227, in_4.world_position, in_4.world_normal, view_z_5);
                shadow_3 = _e230;
            }
            uint _e231 = i_3;
            vec3 _e233 = directional_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2NRUWO2DUNFXGOX(_e231, lighting_input, true);
            light_contrib = _e233;
            vec3 _e235 = light_contrib;
            float _e236 = shadow_3;
            vec3 _e238 = direct_light;
            direct_light = (_e238 + (_e235 * _e236));
        }
    }
    if (true) {
        vec3 _e247 = ambient_lightX_naga_oil_mod_XMJSXM6K7OBRHEOR2MFWWE2LFNZ2AX(in_4.world_position, in_4.N, in_4.V, NdotV_7, _e42, _e60, perceptual_roughness_4, diffuse_occlusion_1);
        vec3 _e249 = indirect_light;
        indirect_light = (_e249 + _e247);
    }
    float _e253 = output_color_2.w;
    emissive_light = (emissive_2.xyz * _e253);
    vec3 _e256 = emissive_light;
    float _e259 = _group_0_binding_0_fs.exposure;
    emissive_light = (_e256 * mix(1.0, _e259, emissive_2.w));
    float _e267 = _group_0_binding_0_fs.exposure;
    vec3 _e268 = transmitted_light;
    vec3 _e269 = direct_light;
    vec3 _e271 = indirect_light;
    vec3 _e274 = emissive_light;
    float _e277 = output_color_2.w;
    output_color_2 = vec4(((_e267 * ((_e268 + _e269) + _e271)) + _e274), _e277);
    vec4 _e279 = output_color_2;
    ClusterableObjectIndexRangesX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX _e281 = clusterable_object_index_ranges_1;
    vec4 _e282 = cluster_debug_visualizationX_naga_oil_mod_XMJSXM6K7OBRHEOR2MNWHK43UMVZGKZC7MZXXE53BOJSAX(_e279, view_z_5, in_4.is_orthographic, _e281, _e113);
    output_color_2 = _e282;
    vec4 _e283 = output_color_2;
    return _e283;
}

vec4 main_pass_post_lighting_processingX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_4, vec4 input_color_1) {
    vec4 output_color_3 = vec4(0.0);
    vec3 output_rgb = vec3(0.0);
    output_color_3 = input_color_1;
    vec4 _e2 = output_color_3;
    ColorGradingX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU5TJMV3QX _e5 = _group_0_binding_0_fs.color_grading;
    vec4 _e6 = tone_mappingX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(_e2, _e5);
    output_color_3 = _e6;
    vec4 _e7 = output_color_3;
    output_rgb = _e7.xyz;
    vec3 _e10 = output_rgb;
    vec3 _e12 = powsafeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(_e10, 0.45454547);
    output_rgb = _e12;
    vec3 _e16 = screen_space_ditherX_naga_oil_mod_XMJSXM6K7MNXXEZK7OBUXAZLMNFXGKOR2ORXW4ZLNMFYHA2LOM4X(pbr_input_4.frag_coord.xy);
    vec3 _e17 = output_rgb;
    output_rgb = (_e17 + _e16);
    vec3 _e19 = output_rgb;
    vec3 _e21 = powsafeX_naga_oil_mod_XMJSXM6K7OJSW4ZDFOI5DU3LBORUHGX(_e19, 2.2);
    output_rgb = _e21;
    vec3 _e22 = output_rgb;
    float _e24 = output_color_3.w;
    output_color_3 = vec4(_e22, _e24);
    vec4 _e26 = output_color_3;
    return _e26;
}

void apply_decalsX_naga_oil_mod_XMJSXM6K7OBRHEOR2MRSWGYLMHI5GG3DVON2GK4TFMQX(inout PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input_5) {
    vec4 base_color_2 = vec4(0.0);
    vec4 emissive_1 = vec4(0.0);
    vec3 Nt = vec3(0.0);
    float metallic_3 = 0.0;
    float perceptual_roughness_3 = 0.0;
    vec4 _e2 = pbr_input_5.world_position;
    vec3 world_position_2 = _e2.xyz;
    vec3 world_normal_2 = pbr_input_5.world_normal;
    vec4 _e7 = pbr_input_5.frag_coord;
    vec2 frag_coord_2 = _e7.xy;
    vec4 _e11 = pbr_input_5.material.base_color;
    base_color_2 = _e11;
    vec4 _e15 = pbr_input_5.material.emissive;
    emissive_1 = _e15;
    vec3 _e18 = pbr_input_5.N;
    Nt = _e18;
    float _e22 = pbr_input_5.material.metallic;
    metallic_3 = _e22;
    float _e26 = pbr_input_5.material.perceptual_roughness;
    perceptual_roughness_3 = _e26;
    vec4 _e30 = base_color_2;
    pbr_input_5.material.base_color = _e30;
    vec4 _e33 = emissive_1;
    pbr_input_5.material.emissive = _e33;
    vec3 _e35 = Nt;
    pbr_input_5.N = normalize(_e35);
    float _e39 = metallic_3;
    pbr_input_5.material.metallic = _e39;
    float _e42 = perceptual_roughness_3;
    pbr_input_5.material.perceptual_roughness = _e42;
    return;
}

void main() {
    VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX vertex_output = VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX(gl_FragCoord, _vs2fs_location0, _vs2fs_location1, _vs2fs_location2, _vs2fs_location6);
    bool is_front = gl_FrontFacing;
    VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX in_ = VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX(vec4(0.0), vec4(0.0), vec3(0.0), vec2(0.0), 0u);
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX pbr_input = PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX(vec4(0.0), vec4(0.0), vec4(0.0), mat3x3(0.0), vec3(0.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, vec2(0.0), 0u, 0.0, 0.0, 0.0, 0.0, 0u, 0u), vec3(0.0), 0.0, vec4(0.0), vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), 0.0, vec3(0.0), vec3(0.0), false, 0u);
    FragmentOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX out_ = FragmentOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX(vec4(0.0));
    in_ = vertex_output;
    VertexOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX _e2 = in_;
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e4 = pbr_input_from_standard_materialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOJQWO3LFNZ2AX(_e2, is_front);
    pbr_input = _e4;
    StandardMaterialX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e9 = pbr_input.material;
    vec4 _e12 = pbr_input.material.base_color;
    vec4 _e13 = alpha_discardX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(_e9, _e12);
    pbr_input.material.base_color = _e13;
    apply_decalsX_naga_oil_mod_XMJSXM6K7OBRHEOR2MRSWGYLMHI5GG3DVON2GK4TFMQX(pbr_input);
    uint _e16 = pbr_input.material.flags;
    if (((_e16 & STANDARD_MATERIAL_FLAGS_UNLIT_BITX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX) == 0u)) {
        PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e23 = pbr_input;
        vec4 _e24 = apply_pbr_lightingX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(_e23);
        out_.color = _e24;
    } else {
        vec4 _e28 = pbr_input.material.base_color;
        out_.color = _e28;
    }
    PbrInputX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3UPFYGK4YX _e30 = pbr_input;
    vec4 _e32 = out_.color;
    vec4 _e33 = main_pass_post_lighting_processingX_naga_oil_mod_XMJSXM6K7OBRHEOR2OBRHEX3GOVXGG5DJN5XHGX(_e30, _e32);
    out_.color = _e33;
    FragmentOutputX_naga_oil_mod_XMJSXM6K7OBRHEOR2MZXXE53BOJSF62LPX _e34 = out_;
    _fs2p_location0 = _e34.color;
    return;
}


/* uam:
Shader failed to compile.
0:3(12): error: extension `GL_EXT_texture_shadow_lod' unsupported in fragment shader


*/
