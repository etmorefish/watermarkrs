#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/favicon-512x512.png")[..],
                )
                .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "图片水印工具",
        native_options,
        Box::new(|cc| {
            // 添加中文字体
            let mut fonts = egui::FontDefinitions::default();
            
            // 尝试加载SimHei字体
            if let Ok(font_data) = std::fs::read("SimHei.ttf") {
                fonts.font_data.insert(
                    "simhei".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                
                // 设置字体族
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "simhei".to_owned());
                    
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("simhei".to_owned());
                    
                cc.egui_ctx.set_fonts(fonts);
            } else {
                log::warn!("无法加载SimHei.ttf字体文件，将使用默认字体");
            }
            
            Ok(Box::new(watermarkrs::WatermarkApp::new(cc)))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    // 添加中文字体（Web版本）
                    let mut fonts = egui::FontDefinitions::default();
                    
                    // 对于Web版本，我们可以使用系统字体或嵌入字体
                    // 这里我们添加一个中文字体作为后备
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "Microsoft YaHei".to_owned());
                        
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("Microsoft YaHei".to_owned());
                    
                    // 添加更多中文字体作为后备
                    let chinese_fonts = vec![
                        "Microsoft YaHei",
                        "SimHei",
                        "NSimSun",
                        "FangSong",
                        "KaiTi",
                        "STHeiti",
                        "STKaiti",
                        "STSong",
                        "STFangsong",
                        "Arial Unicode MS",
                        "sans-serif",
                    ];
                    
                    for font in chinese_fonts {
                        fonts
                            .families
                            .entry(egui::FontFamily::Proportional)
                            .or_default()
                            .push(font.to_owned());
                    }
                    
                    cc.egui_ctx.set_fonts(fonts);
                    
                    Ok(Box::new(watermarkrs::WatermarkApp::new(cc)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
