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
    Err(anyhow::anyhow!("GUI feature not enabled. Compile with --features gui"))
}

#[cfg(feature = "gui")]
pub fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([500.0, 400.0])
            .with_title("UDisks2 ISO 挂载工具"),
        ..Default::default()
    };

    let app = GuiApp::new();
    eframe::run_native(
        "UDisks2 ISO 挂载工具",
        options,
        Box::new(move |cc| {
            // 使用系统中文字体
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

#[cfg(feature = "gui")]
fn setup_fonts(ctx: &egui::Context) {
    use egui::FontDefinitions;

    let mut fonts = FontDefinitions::default();

            // 在Linux系统中尝试加载系统中文字体
    if let Ok(font_data) = std::fs::read("/usr/share/fonts/opentype/source-han-cjk/SourceHanSansSC-Regular.otf") {
        fonts.font_data.insert(
            "source_han_sans".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "source_han_sans".to_owned());
    }

    ctx.set_fonts(fonts);
}