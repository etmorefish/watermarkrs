//! 水印应用主界面

use eframe::egui;
use image::DynamicImage;
use std::sync::Arc;

use crate::watermark::{Position, WatermarkConfig};
use crate::image_ops::{add_watermark, get_image_info};

/// 应用状态
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WatermarkApp {
    // 图像状态（跳过序列化，因为DynamicImage不能序列化）
    #[serde(skip)]
    base_images: Vec<(String, Arc<DynamicImage>)>, // (文件名, 图像)
    #[serde(skip)]
    logo_images: Vec<(String, Arc<DynamicImage>)>, // Logo缓存
    selected_base_index: Option<usize>,
    selected_logo_index: Option<usize>,
    
    // 水印配置
    config: WatermarkConfig,
    
    // 预览状态（跳过序列化）
    #[serde(skip)]
    preview_image: Option<Arc<DynamicImage>>,
    #[serde(skip)]
    preview_loading: bool,
    
    // UI状态
    error_message: Option<String>,
    success_message: Option<String>,
    
    // 文件处理状态
    #[serde(skip)]
    file_dialog_open: bool,
    #[serde(skip)]
    logo_dialog_open: bool,
    #[serde(skip)]
    save_dialog_open: bool,
}

impl Default for WatermarkApp {
    fn default() -> Self {
        Self {
            base_images: Vec::new(),
            logo_images: Vec::new(),
            selected_base_index: None,
            selected_logo_index: None,
            config: WatermarkConfig::default(),
            preview_image: None,
            preview_loading: false,
            error_message: None,
            success_message: None,
            file_dialog_open: false,
            logo_dialog_open: false,
            save_dialog_open: false,
        }
    }
}

impl WatermarkApp {
    /// 创建新应用
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载保存的状态
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
    
    /// 添加基础图像
    fn add_base_image(&mut self, name: String, image: DynamicImage) {
        self.base_images.push((name, Arc::new(image)));
        if self.selected_base_index.is_none() && !self.base_images.is_empty() {
            self.selected_base_index = Some(0);
        }
    }
    
    /// 添加Logo图像
    fn add_logo_image(&mut self, name: String, image: DynamicImage) {
        self.logo_images.push((name, Arc::new(image)));
        if self.selected_logo_index.is_none() && !self.logo_images.is_empty() {
            self.selected_logo_index = Some(0);
        }
    }
    
    /// 获取当前选中的基础图像
    fn current_base_image(&self) -> Option<&DynamicImage> {
        self.selected_base_index
            .and_then(|idx| self.base_images.get(idx))
            .map(|(_, img)| img.as_ref())
    }
    
    /// 获取当前选中的Logo图像
    fn current_logo_image(&self) -> Option<&DynamicImage> {
        self.selected_logo_index
            .and_then(|idx| self.logo_images.get(idx))
            .map(|(_, img)| img.as_ref())
    }
    
    /// 更新预览
    fn update_preview(&mut self) {
        // 先获取图像引用，避免借用冲突
        let base_image = self.current_base_image().map(|img| img.clone());
        let logo_image = self.current_logo_image().map(|img| img.clone());
        
        if let (Some(base), Some(logo)) = (base_image, logo_image) {
            self.preview_loading = true;
            
            // 在实际应用中，这里应该使用异步处理
            // 为了简单起见，我们直接处理
            match add_watermark(&base, &logo, &self.config) {
                Ok(result) => {
                    self.preview_image = Some(Arc::new(result));
                    self.error_message = None;
                }
                Err(err) => {
                    self.error_message = Some(err);
                    self.preview_image = None;
                }
            }
            
            self.preview_loading = false;
        } else {
            self.preview_image = None;
        }
    }
    
    /// 保存当前预览图像
    fn save_preview_image(&self) -> Result<(), String> {
        if let Some(_preview) = &self.preview_image {
            let format = if let Some((name, _)) = self.selected_base_index
                .and_then(|idx| self.base_images.get(idx))
            {
                if name.to_lowercase().ends_with(".jpg") || name.to_lowercase().ends_with(".jpeg") {
                    "jpg"
                } else {
                    "png"
                }
            } else {
                "png"
            };
            
            let filename = if let Some((name, _)) = self.selected_base_index
                .and_then(|idx| self.base_images.get(idx))
            {
                let base_name = name.rsplit('.').next().unwrap_or(name);
                format!("{}_watermarked.{}", base_name, format)
            } else {
                format!("watermarked.{}", format)
            };
            
            // 在实际应用中，这里应该使用文件对话框
            // 为了演示，我们只打印信息
            println!("保存图像: {}", filename);
            
            Ok(())
        } else {
            Err("没有预览图像可保存".to_string())
        }
    }
    
    /// 显示错误消息
    fn show_error(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::RED, "错误:");
        ui.label(message);
    }
    
    /// 显示成功消息
    fn show_success(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::GREEN, "成功:");
        ui.label(message);
    }
}

impl eframe::App for WatermarkApp {
    /// 保存状态
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    
    /// 更新UI
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 顶部菜单栏
        egui::TopBottomPanel::top("menu_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("打开图片").clicked() {
                        self.file_dialog_open = true;
                        ui.close_menu();
                    }
                    
                    if ui.button("打开Logo").clicked() {
                        self.logo_dialog_open = true;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("保存图片").clicked() {
                        if let Err(err) = self.save_preview_image() {
                            self.error_message = Some(err);
                        } else {
                            self.success_message = Some("图片保存成功".to_string());
                        }
                        ui.close_menu();
                    }
                    
                    if !cfg!(target_arch = "wasm32") {
                        ui.separator();
                        if ui.button("退出").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                
                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        // 显示关于对话框
                        ui.close_menu();
                    }
                });
            });
        });
        
        // 主内容区域
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("图片水印工具");
            
            // 显示消息
            if let Some(error) = self.error_message.as_ref() {
                self.show_error(ui, error);
            }
            
            if let Some(success) = self.success_message.as_ref() {
                self.show_success(ui, success);
            }
            
            ui.separator();
            
            // 两列布局
            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    // 左侧：图像选择和预览
                    ui.vertical(|ui| {
                        ui.heading("图像选择");
                        
                        // 基础图像选择
                        ui.group(|ui| {
                            ui.label("基础图像:");
                            
                            if self.base_images.is_empty() {
                                ui.label("没有图像");
                                if ui.button("选择图像").clicked() {
                                    self.file_dialog_open = true;
                                }
                            } else {
                                egui::ComboBox::from_label("选择图像")
                                    .selected_text(
                                        self.selected_base_index
                                            .and_then(|idx| self.base_images.get(idx))
                                            .map(|(name, _)| name.as_str())
                                            .unwrap_or("未选择")
                                    )
                                    .show_ui(ui, |ui| {
                                        for (i, (name, _)) in self.base_images.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.selected_base_index,
                                                Some(i),
                                                name,
                                            );
                                        }
                                    });
                                
                                if let Some(image) = self.current_base_image() {
                                    let (width, height, format) = get_image_info(image);
                                    ui.label(format!("尺寸: {}x{}", width, height));
                                    ui.label(format!("格式: {}", format));
                                }
                            }
                        });
                        
                        ui.add_space(10.0);
                        
                        // Logo图像选择
                        ui.group(|ui| {
                            ui.label("Logo图像:");
                            
                            if self.logo_images.is_empty() {
                                ui.label("没有Logo");
                                if ui.button("选择Logo").clicked() {
                                    self.logo_dialog_open = true;
                                }
                            } else {
                                egui::ComboBox::from_label("选择Logo")
                                    .selected_text(
                                        self.selected_logo_index
                                            .and_then(|idx| self.logo_images.get(idx))
                                            .map(|(name, _)| name.as_str())
                                            .unwrap_or("未选择")
                                    )
                                    .show_ui(ui, |ui| {
                                        for (i, (name, _)) in self.logo_images.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.selected_logo_index,
                                                Some(i),
                                                name,
                                            );
                                        }
                                    });
                                
                                if let Some(image) = self.current_logo_image() {
                                    let (width, height, format) = get_image_info(image);
                                    ui.label(format!("尺寸: {}x{}", width, height));
                                    ui.label(format!("格式: {}", format));
                                }
                            }
                        });
                        
                        ui.add_space(10.0);
                        
                        // 预览按钮
                        if ui.button("更新预览").clicked() {
                            self.update_preview();
                        }
                    });
                    
                    ui.end_row();
                    
                    // 右侧：水印设置
                    ui.vertical(|ui| {
                        ui.heading("水印设置");
                        
                        ui.group(|ui| {
                            // 位置选择
                            ui.label("位置:");
                            egui::Grid::new("position_grid")
                                .num_columns(3)
                                .spacing([5.0, 5.0])
                                .show(ui, |ui| {
                                    let positions = [
                                        (Position::TopLeft, "左上"),
                                        (Position::TopCenter, "上中"),
                                        (Position::TopRight, "右上"),
                                        (Position::MiddleLeft, "左中"),
                                        (Position::Center, "居中"),
                                        (Position::MiddleRight, "右中"),
                                        (Position::BottomLeft, "左下"),
                                        (Position::BottomCenter, "下中"),
                                        (Position::BottomRight, "右下"),
                                    ];
                                    
                                    for (pos, label) in positions {
                                        if ui
                                            .selectable_label(
                                                self.config.position == pos,
                                                label,
                                            )
                                            .clicked()
                                        {
                                            self.config.position = pos;
                                            self.update_preview();
                                        }
                                        
                                        if pos == Position::MiddleRight {
                                            ui.end_row();
                                        }
                                    }
                                });
                            
                            ui.add_space(10.0);
                            
                            // 透明度设置
                            ui.horizontal(|ui| {
                                ui.label("透明度:");
                                if ui
                                    .add(
                                        egui::Slider::new(&mut self.config.opacity, 0.0..=1.0)
                                            .step_by(0.05)
                                            .show_value(false),
                                    )
                                    .changed()
                                {
                                    self.update_preview();
                                }
                                ui.label(format!("{:.2}", self.config.opacity));
                            });
                            
                            // 缩放比例设置
                            ui.horizontal(|ui| {
                                ui.label("缩放比例:");
                                if ui
                                    .add(
                                        egui::Slider::new(&mut self.config.scale, 0.05..=0.5)
                                            .step_by(0.05)
                                            .show_value(false),
                                    )
                                    .changed()
                                {
                                    self.update_preview();
                                }
                                ui.label(format!("{:.2}", self.config.scale));
                            });
                            
                            // 边距设置
                            ui.horizontal(|ui| {
                                ui.label("边距:");
                                let mut margin = self.config.margin as f64;
                                if ui
                                    .add(
                                        egui::Slider::new(&mut margin, 0.0..=100.0)
                                            .step_by(5.0),
                                    )
                                    .changed()
                                {
                                    self.config.margin = margin as u32;
                                    self.update_preview();
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 预览区域
                        ui.heading("预览");
                        
                        if self.preview_loading {
                            ui.spinner();
                        } else if let Some(preview) = &self.preview_image {
                            let (width, height, _) = get_image_info(preview);
                            
                            // 计算适合UI的显示尺寸
                            let max_size = 300.0_f32;
                            let aspect_ratio = width as f32 / height as f32;
                            
                            let display_width = if width > height {
                                max_size
                            } else {
                                max_size * aspect_ratio
                            };
                            
                            let display_height = if width > height {
                                max_size / aspect_ratio
                            } else {
                                max_size
                            };
                            
                            // 显示预览图像
                            ui.image(
                                egui::load::SizedTexture::new(
                                    egui::TextureId::default(),
                                    egui::vec2(display_width, display_height),
                                ),
                            );
                            
                            ui.label(format!("预览尺寸: {}x{}", width, height));
                            
                            // 保存按钮
                            if ui.button("保存图片").clicked() {
                                if let Err(err) = self.save_preview_image() {
                                    self.error_message = Some(err);
                                } else {
                                    self.success_message = Some("图片保存成功".to_string());
                                }
                            }
                        } else {
                            ui.label("没有预览图像");
                        }
                    });
                });
            
            ui.separator();
            
            // 状态栏
            ui.horizontal(|ui| {
                ui.label("状态:");
                
                if self.base_images.is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, "请选择基础图像");
                } else if self.logo_images.is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, "请选择Logo图像");
                } else if self.preview_image.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "就绪");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "等待配置");
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        "水印工具 v0.1.0",
                        "https://github.com/yourusername/watermarkrs",
                    );
                });
            });
        });
        
        // 处理文件对话框
        if self.file_dialog_open {
            // 在实际应用中，这里应该使用rfd打开文件对话框
            // 为了演示，我们只是关闭对话框
            self.file_dialog_open = false;
            
            // 模拟添加测试图像
            if self.base_images.is_empty() {
                // 创建测试图像
                let test_image = DynamicImage::new_rgb8(800, 600);
                self.add_base_image("测试图像.jpg".to_string(), test_image);
                self.success_message = Some("已添加测试图像".to_string());
            }
        }
        
        if self.logo_dialog_open {
            self.logo_dialog_open = false;
            
            // 模拟添加测试Logo
            if self.logo_images.is_empty() {
                // 创建测试Logo
                let test_logo = DynamicImage::new_rgba8(200, 100);
                self.add_logo_image("测试Logo.png".to_string(), test_logo);
                self.success_message = Some("已添加测试Logo".to_string());
            }
        }
    }
}
