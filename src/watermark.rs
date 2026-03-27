//! 水印配置和位置计算模块

use serde::{Deserialize, Serialize};

/// 九宫格位置枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    /// 左上
    TopLeft,
    /// 上中
    TopCenter,
    /// 右上
    TopRight,
    /// 左中
    MiddleLeft,
    /// 居中
    Center,
    /// 右中
    MiddleRight,
    /// 左下
    BottomLeft,
    /// 下中
    BottomCenter,
    /// 右下
    BottomRight,
}

impl Position {
    /// 获取所有可用位置
    pub fn all() -> Vec<Position> {
        vec![
            Position::TopLeft,
            Position::TopCenter,
            Position::TopRight,
            Position::MiddleLeft,
            Position::Center,
            Position::MiddleRight,
            Position::BottomLeft,
            Position::BottomCenter,
            Position::BottomRight,
        ]
    }

    /// 获取位置的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Position::TopLeft => "左上",
            Position::TopCenter => "上中",
            Position::TopRight => "右上",
            Position::MiddleLeft => "左中",
            Position::Center => "居中",
            Position::MiddleRight => "右中",
            Position::BottomLeft => "左下",
            Position::BottomCenter => "下中",
            Position::BottomRight => "右下",
        }
    }

    /// 计算水印在基础图像上的位置
    /// 返回 (x, y) 坐标
    pub fn calculate_position(
        &self,
        base_width: u32,
        base_height: u32,
        watermark_width: u32,
        watermark_height: u32,
        margin: u32,
    ) -> (u32, u32) {
        let margin = margin as i32;
        let base_width = base_width as i32;
        let base_height = base_height as i32;
        let watermark_width = watermark_width as i32;
        let watermark_height = watermark_height as i32;

        let (x, y) = match self {
            Position::TopLeft => (margin, margin),
            Position::TopCenter => (
                (base_width - watermark_width) / 2,
                margin,
            ),
            Position::TopRight => (
                base_width - watermark_width - margin,
                margin,
            ),
            Position::MiddleLeft => (
                margin,
                (base_height - watermark_height) / 2,
            ),
            Position::Center => (
                (base_width - watermark_width) / 2,
                (base_height - watermark_height) / 2,
            ),
            Position::MiddleRight => (
                base_width - watermark_width - margin,
                (base_height - watermark_height) / 2,
            ),
            Position::BottomLeft => (
                margin,
                base_height - watermark_height - margin,
            ),
            Position::BottomCenter => (
                (base_width - watermark_width) / 2,
                base_height - watermark_height - margin,
            ),
            Position::BottomRight => (
                base_width - watermark_width - margin,
                base_height - watermark_height - margin,
            ),
        };

        (x.max(0) as u32, y.max(0) as u32)
    }
}

/// 水印配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    /// 水印位置
    pub position: Position,
    /// 透明度 (0.0 ~ 1.0)
    pub opacity: f32,
    /// 缩放比例 (相对于原图宽度，如 0.1 ~ 0.5)
    pub scale: f32,
    /// 边距 (像素)
    pub margin: u32,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            position: Position::BottomRight,
            opacity: 0.7,
            scale: 0.2,
            margin: 20,
        }
    }
}

impl WatermarkConfig {
    /// 创建新的水印配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 验证配置是否有效
    pub fn is_valid(&self) -> bool {
        self.opacity >= 0.0 && self.opacity <= 1.0 && self.scale > 0.0 && self.scale <= 1.0
    }
}
