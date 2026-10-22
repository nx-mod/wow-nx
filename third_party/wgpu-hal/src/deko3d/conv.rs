//! wgpu formats and dimensions in deko3d's terms (the same table as dawn-nx's UtilsDeko3d).

use super::ffi::{DkImageFormat, DkImageType};

pub(super) fn formato(format: wgt::TextureFormat) -> Option<DkImageFormat> {
    use wgt::TextureFormat as T;
    use DkImageFormat as F;
    Some(match format {
        T::R8Unorm => F::DkImageFormat_R8_Unorm,
        T::R8Snorm => F::DkImageFormat_R8_Snorm,
        T::R8Uint => F::DkImageFormat_R8_Uint,
        T::R8Sint => F::DkImageFormat_R8_Sint,
        T::R16Unorm => F::DkImageFormat_R16_Unorm,
        T::R16Snorm => F::DkImageFormat_R16_Snorm,
        T::R16Uint => F::DkImageFormat_R16_Uint,
        T::R16Sint => F::DkImageFormat_R16_Sint,
        T::R16Float => F::DkImageFormat_R16_Float,
        T::Rg8Unorm => F::DkImageFormat_RG8_Unorm,
        T::Rg8Snorm => F::DkImageFormat_RG8_Snorm,
        T::Rg8Uint => F::DkImageFormat_RG8_Uint,
        T::Rg8Sint => F::DkImageFormat_RG8_Sint,
        T::R32Float => F::DkImageFormat_R32_Float,
        T::R32Uint => F::DkImageFormat_R32_Uint,
        T::R32Sint => F::DkImageFormat_R32_Sint,
        T::Rg16Unorm => F::DkImageFormat_RG16_Unorm,
        T::Rg16Snorm => F::DkImageFormat_RG16_Snorm,
        T::Rg16Uint => F::DkImageFormat_RG16_Uint,
        T::Rg16Sint => F::DkImageFormat_RG16_Sint,
        T::Rg16Float => F::DkImageFormat_RG16_Float,
        T::Rgba8Unorm => F::DkImageFormat_RGBA8_Unorm,
        T::Rgba8UnormSrgb => F::DkImageFormat_RGBA8_Unorm_sRGB,
        T::Rgba8Snorm => F::DkImageFormat_RGBA8_Snorm,
        T::Rgba8Uint => F::DkImageFormat_RGBA8_Uint,
        T::Rgba8Sint => F::DkImageFormat_RGBA8_Sint,
        T::Bgra8Unorm => F::DkImageFormat_BGRA8_Unorm,
        T::Bgra8UnormSrgb => F::DkImageFormat_BGRA8_Unorm_sRGB,
        T::Rgb10a2Uint => F::DkImageFormat_RGB10A2_Uint,
        T::Rgb10a2Unorm => F::DkImageFormat_RGB10A2_Unorm,
        T::Rg11b10Ufloat => F::DkImageFormat_RG11B10_Float,
        T::Rgb9e5Ufloat => F::DkImageFormat_E5BGR9_Float,
        T::Rg32Float => F::DkImageFormat_RG32_Float,
        T::Rg32Uint => F::DkImageFormat_RG32_Uint,
        T::Rg32Sint => F::DkImageFormat_RG32_Sint,
        T::Rgba16Unorm => F::DkImageFormat_RGBA16_Unorm,
        T::Rgba16Snorm => F::DkImageFormat_RGBA16_Snorm,
        T::Rgba16Uint => F::DkImageFormat_RGBA16_Uint,
        T::Rgba16Sint => F::DkImageFormat_RGBA16_Sint,
        T::Rgba16Float => F::DkImageFormat_RGBA16_Float,
        T::Rgba32Float => F::DkImageFormat_RGBA32_Float,
        T::Rgba32Uint => F::DkImageFormat_RGBA32_Uint,
        T::Rgba32Sint => F::DkImageFormat_RGBA32_Sint,
        T::Stencil8 => F::DkImageFormat_S8,
        T::Depth16Unorm => F::DkImageFormat_Z16,
        T::Depth24Plus => F::DkImageFormat_Z24X8,
        T::Depth24PlusStencil8 => F::DkImageFormat_Z24S8,
        T::Depth32Float => F::DkImageFormat_ZF32,
        T::Depth32FloatStencil8 => F::DkImageFormat_ZF32_X24S8,
        T::Bc1RgbaUnorm => F::DkImageFormat_RGBA_BC1,
        T::Bc1RgbaUnormSrgb => F::DkImageFormat_RGBA_BC1_sRGB,
        T::Bc2RgbaUnorm => F::DkImageFormat_RGBA_BC2,
        T::Bc2RgbaUnormSrgb => F::DkImageFormat_RGBA_BC2_sRGB,
        T::Bc3RgbaUnorm => F::DkImageFormat_RGBA_BC3,
        T::Bc3RgbaUnormSrgb => F::DkImageFormat_RGBA_BC3_sRGB,
        T::Bc4RUnorm => F::DkImageFormat_R_BC4_Unorm,
        T::Bc4RSnorm => F::DkImageFormat_R_BC4_Snorm,
        T::Bc5RgUnorm => F::DkImageFormat_RG_BC5_Unorm,
        T::Bc5RgSnorm => F::DkImageFormat_RG_BC5_Snorm,
        T::Bc6hRgbUfloat => F::DkImageFormat_RGBA_BC6H_UF16_Float,
        T::Bc6hRgbFloat => F::DkImageFormat_RGBA_BC6H_SF16_Float,
        T::Bc7RgbaUnorm => F::DkImageFormat_RGBA_BC7_Unorm,
        T::Bc7RgbaUnormSrgb => F::DkImageFormat_RGBA_BC7_Unorm_sRGB,
        _ => return None,
    })
}

/// The type an image is laid out as. Cube textures are 2D arrays; their views say cube.
pub(super) fn tipo_imagen(desc: &crate::TextureDescriptor) -> DkImageType {
    use DkImageType as I;
    let capas = desc.size.depth_or_array_layers;
    match desc.dimension {
        wgt::TextureDimension::D1 if capas > 1 => I::DkImageType_1DArray,
        wgt::TextureDimension::D1 => I::DkImageType_1D,
        wgt::TextureDimension::D3 => I::DkImageType_3D,
        wgt::TextureDimension::D2 => match (desc.sample_count > 1, capas > 1) {
            (true, true) => I::DkImageType_2DMSArray,
            (true, false) => I::DkImageType_2DMS,
            (false, true) => I::DkImageType_2DArray,
            (false, false) => I::DkImageType_2D,
        },
    }
}

pub(super) fn tipo_vista(dim: wgt::TextureViewDimension, multimuestra: bool) -> DkImageType {
    use wgt::TextureViewDimension as V;
    use DkImageType as I;
    match dim {
        V::D1 => I::DkImageType_1D,
        V::D2 if multimuestra => I::DkImageType_2DMS,
        V::D2 => I::DkImageType_2D,
        V::D2Array if multimuestra => I::DkImageType_2DMSArray,
        V::D2Array => I::DkImageType_2DArray,
        V::Cube => I::DkImageType_Cubemap,
        V::CubeArray => I::DkImageType_CubemapArray,
        V::D3 => I::DkImageType_3D,
    }
}

pub(super) fn muestras(cuenta: u32) -> super::ffi::DkMsMode {
    use super::ffi::DkMsMode as M;
    match cuenta {
        2 => M::DkMsMode_2x,
        4 => M::DkMsMode_4x,
        8 => M::DkMsMode_8x,
        _ => M::DkMsMode_1x,
    }
}
