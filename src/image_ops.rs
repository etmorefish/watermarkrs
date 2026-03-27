//! 图像处理核心算法

use image::{DynamicImage, RgbaImage};
use crate::watermark::WatermarkConfig;

/// 添加水印到图像
///
/// # 参数
/// - `base_image`: 基础图像
/// - `logo_image`: Logo图像
/// - `config`: 水印配置
///
/// # 返回
/// 添加水印后的新图像
pub fn add_watermark(
    base_image: &DynamicImage,
    logo_image: &DynamicImage,
    config: &WatermarkConfig,
) -> Result<DynamicImage, String> {
    if !config.is_valid() {
        return Err("无效的水印配置".to_string());
    }

    // 计算缩放后的Logo尺寸
    let base_width = base_image.width();
    let scaled_logo_width = (base_width as f32 * config.scale) as u32;
    
    // 保持Logo的宽高比
    let logo_width = logo_image.width();
    let logo_height = logo_image.height();
    let aspect_ratio = logo_height as f32 / logo_width as f32;
    let scaled_logo_height = (scaled_logo_width as f32 * aspect_ratio) as u32;

    // 缩放Logo图像
    let scaled_logo = logo_image.resize(
        scaled_logo_width,
        scaled_logo_height,
        image::imageops::FilterType::Lanczos3,
    );

    // 计算水印位置
    let (x, y) = config.position.calculate_position(
        base_width,
        base_image.height(),
        scaled_logo_width,
        scaled_logo_height,
        config.margin,
    );

    // 创建结果图像（复制基础图像）
    let mut result = base_image.to_rgba8();

    // 应用水印
    apply_watermark_to_image(
        &mut result,
        &scaled_logo.to_rgba8(),
        x,
        y,
        config.opacity,
    );

    Ok(DynamicImage::ImageRgba8(result))
}

/// 将水印应用到图像上（使用alpha blending）
fn apply_watermark_to_image(
    base: &mut RgbaImage,
    watermark: &RgbaImage,
    x: u32,
    y: u32,
    opacity: f32,
) {
    let base_width = base.width();
    let base_height = base.height();
    let watermark_width = watermark.width();
    let watermark_height = watermark.height();

    // 确保水印在图像范围内
    let end_x = (x + watermark_width).min(base_width);
    let end_y = (y + watermark_height).min(base_height);
    let actual_width = end_x.saturating_sub(x);
    let actual_height = end_y.saturating_sub(y);

    for wy in 0..actual_height {
        for wx in 0..actual_width {
            let base_x = x + wx;
            let base_y = y + wy;
            
            let watermark_pixel = watermark.get_pixel(wx, wy);
            let base_pixel = base.get_pixel(base_x, base_y);

            // 计算混合后的像素
            let blended = blend_pixels(base_pixel, watermark_pixel, opacity);
            base.put_pixel(base_x, base_y, blended);
        }
    }
}

/// 混合两个像素（使用alpha blending）
fn blend_pixels(base: &image::Rgba<u8>, overlay: &image::Rgba<u8>, opacity: f32) -> image::Rgba<u8> {
    let base_alpha = base[3] as f32 / 255.0;
    let overlay_alpha = overlay[3] as f32 / 255.0 * opacity;
    
    // 计算最终alpha
    let final_alpha = overlay_alpha + base_alpha * (1.0 - overlay_alpha);
    
    if final_alpha == 0.0 {
        return image::Rgba([0, 0, 0, 0]);
    }

    let blend_channel = |base_channel: u8, overlay_channel: u8| {
        let base_val = base_channel as f32 / 255.0;
        let overlay_val = overlay_channel as f32 / 255.0;
        
        let blended = (overlay_val * overlay_alpha + base_val * base_alpha * (1.0 - overlay_alpha)) / final_alpha;
        (blended * 255.0).round() as u8
    };

    image::Rgba([
        blend_channel(base[0], overlay[0]),
        blend_channel(base[1], overlay[1]),
        blend_channel(base[2], overlay[2]),
        (final_alpha * 255.0).round() as u8,
    ])
}

/// 从字节加载图像
pub fn load_image_from_bytes(data: &[u8]) -> Result<DynamicImage, String> {
    image::load_from_memory(data)
        .map_err(|e| format!("加载图像失败: {}", e))
}

/// 将图像保存为字节（PNG格式）
pub fn save_image_as_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
        .map_err(|e| format!("保存图像失败: {}", e))?;
    Ok(buffer)
}

/// 将图像保存为字节（JPG格式）
pub fn save_image_as_jpg(image: &DynamicImage, _quality: u8) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    
    // 使用更简单的方法保存JPG
    image
        .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Jpeg)
        .map_err(|e| format!("保存JPG图像失败: {}", e))?;
    Ok(buffer)
}

/// 获取图像信息
pub fn get_image_info(image: &DynamicImage) -> (u32, u32, String) {
    let width = image.width();
    let height = image.height();
    let format = match image {
        DynamicImage::ImageLuma8(_) => "灰度8位",
        DynamicImage::ImageLumaA8(_) => "灰度+Alpha8位",
        DynamicImage::ImageRgb8(_) => "RGB8位",
        DynamicImage::ImageRgba8(_) => "RGBA8位",
        DynamicImage::ImageLuma16(_) => "灰度16位",
        DynamicImage::ImageLumaA16(_) => "灰度+Alpha16位",
        DynamicImage::ImageRgb16(_) => "RGB16位",
        DynamicImage::ImageRgba16(_) => "RGBA16位",
        DynamicImage::ImageRgb32F(_) => "RGB32位浮点",
        DynamicImage::ImageRgba32F(_) => "RGBA32位浮点",
        _ => "未知格式",
    };
    
    (width, height, format.to_string())
}
