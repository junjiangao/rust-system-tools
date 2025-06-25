#[cfg(feature = "gui")]
use crate::config::{AppConfig, FontLoader};
#[cfg(feature = "gui")]
use anyhow::Result;
#[cfg(feature = "gui")]
use egui::*;
#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
pub struct GuiApp {
    iso_path: Option<PathBuf>,
    status_message: String,
    is_mounted: bool,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "gui")]
impl GuiApp {
    pub fn new() -> Self {
        Self {
            iso_path: None,
            status_message: "Ready".to_string(),
            is_mounted: false,
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("UDisks2 ISO Mounter");

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("ISO路径:");
                if let Some(ref path) = self.iso_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("未选择文件");
                }
            });

            if ui.button("选择ISO文件").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("ISO文件", &["iso"])
                    .pick_file()
                {
                    self.iso_path = Some(path);
                    self.status_message = "ISO文件已选择".to_string();
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
            });

            ui.separator();

            if ui.button("Mount ISO").clicked() {
                self.status_message = "Mounting...".to_string();
                // 这里可以集成实际的挂载逻辑
                self.is_mounted = true;
                self.status_message = "Mounted successfully".to_string();
            }

            if self.is_mounted && ui.button("Unmount ISO").clicked() {
                self.status_message = "Unmounting...".to_string();
                // 这里可以集成实际的卸载逻辑
                self.is_mounted = false;
                self.status_message = "Unmounted successfully".to_string();
            }

            ui.separator();

            if ui.button("Exit").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        });
    }
}

#[cfg(not(feature = "gui"))]
use std::path::PathBuf;

#[cfg(not(feature = "gui"))]
pub struct GuiApp;

#[cfg(not(feature = "gui"))]
impl GuiApp {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "gui"))]
pub fn run_gui() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "GUI feature not enabled. Compile with --features gui"
    ))
}

#[cfg(feature = "gui")]
pub fn run_gui() -> Result<()> {
    // 加载配置
    let config = AppConfig::load()?;

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([config.gui.window_width, config.gui.window_height])
            .with_title("UDisks2 ISO 挂载工具"),
        ..Default::default()
    };

    let app = GuiApp::new();
    eframe::run_native(
        "UDisks2 ISO 挂载工具",
        options,
        Box::new(move |cc| {
            // 使用配置中的字体
            setup_fonts(&cc.egui_ctx, &config)?;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

#[cfg(feature = "gui")]
fn setup_fonts(ctx: &egui::Context, config: &AppConfig) -> Result<()> {
    use egui::FontDefinitions;

    let mut fonts = FontDefinitions::default();
    let font_loader = FontLoader::new();

    // 获取配置的字体族列表
    let _font_families = config.get_font_families();

    // 尝试分组加载字体
    let mut loaded_fonts = Vec::new();

    // 加载中文字体
    if let Some(font_data) = font_loader.find_font_data(&config.gui.font_families.chinese) {
        let font_id = "chinese_font".to_string();
        fonts.font_data.insert(
            font_id.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        loaded_fonts.push(font_id);
    }

    // 加载英文字体
    if let Some(font_data) = font_loader.find_font_data(&config.gui.font_families.english) {
        let font_id = "english_font".to_string();
        fonts.font_data.insert(
            font_id.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        loaded_fonts.push(font_id);
    }

    // 加载fallback字体
    if let Some(font_data) = font_loader.find_font_data(&config.gui.font_families.fallback) {
        let font_id = "fallback_font".to_string();
        fonts.font_data.insert(
            font_id.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        loaded_fonts.push(font_id);
    }

    // 设置字体族优先级
    if !loaded_fonts.is_empty() {
        // 为Proportional字体族设置优先级
        let proportional_fonts = fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default();

        // 清空默认字体，按优先级添加加载的字体
        proportional_fonts.clear();
        for font_id in &loaded_fonts {
            proportional_fonts.push(font_id.clone());
        }

        // 为Monospace字体族也设置相同的字体
        let monospace_fonts = fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default();

        for font_id in &loaded_fonts {
            monospace_fonts.push(font_id.clone());
        }

        println!("Loaded {} fonts: {:?}", loaded_fonts.len(), loaded_fonts);
    } else {
        println!("No fonts could be loaded from config, using default system fonts");
    }

    ctx.set_fonts(fonts);
    Ok(())
}
