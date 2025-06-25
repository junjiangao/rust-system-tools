use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct FontLoader {
    db: fontdb::Database,
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl FontLoader {
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        Self { db }
    }

    /// 根据字体族名称查找字体数据
    pub fn find_font_data(&self, family_names: &[String]) -> Option<Vec<u8>> {
        for family_name in family_names {
            if let Some(id) = self.db.query(&fontdb::Query {
                families: &[fontdb::Family::Name(family_name)],
                ..Default::default()
            }) {
                if let Some(_face) = self.db.face(id) {
                    // 尝试从face中获取数据
                    if let Some(data) = self.db.with_face_data(id, |font_data, face_index| {
                        println!("Found font: {} (face index: {})", family_name, face_index);
                        Some(font_data.to_vec())
                    }) {
                        return data;
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub gui: GuiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    #[serde(default = "default_font_families")]
    pub font_families: FontFamilies,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilies {
    #[serde(default = "default_chinese_fonts")]
    pub chinese: Vec<String>,
    #[serde(default = "default_english_fonts")]
    pub english: Vec<String>,
    #[serde(default = "default_fallback_fonts")]
    pub fallback: Vec<String>,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            font_families: default_font_families(),
            font_size: default_font_size(),
            window_width: default_window_width(),
            window_height: default_window_height(),
        }
    }
}

impl Default for FontFamilies {
    fn default() -> Self {
        Self {
            chinese: default_chinese_fonts(),
            english: default_english_fonts(),
            fallback: default_fallback_fonts(),
        }
    }
}

fn default_font_families() -> FontFamilies {
    FontFamilies::default()
}

fn default_chinese_fonts() -> Vec<String> {
    vec![
        // Linux
        "Source Han Sans SC".to_string(),
        "Noto Sans CJK SC".to_string(),
        "WenQuanYi Zen Hei".to_string(),
        // macOS
        "PingFang SC".to_string(),
        "Hiragino Sans GB".to_string(),
        // Windows
        "Microsoft YaHei".to_string(),
        "SimSun".to_string(),
        "SimHei".to_string(),
    ]
}

fn default_english_fonts() -> Vec<String> {
    vec![
        // 常见系统字体
        "Inter".to_string(),
        "Segoe UI".to_string(),
        "San Francisco".to_string(),
        "Helvetica Neue".to_string(),
        "Arial".to_string(),
        "Liberation Sans".to_string(),
        "DejaVu Sans".to_string(),
    ]
}

fn default_fallback_fonts() -> Vec<String> {
    vec![
        "Noto Sans".to_string(),
        "Liberation Sans".to_string(),
        "DejaVu Sans".to_string(),
        "Arial".to_string(),
        "sans-serif".to_string(),
    ]
}

fn default_font_size() -> f32 {
    14.0
}

fn default_window_width() -> f32 {
    500.0
}

fn default_window_height() -> f32 {
    400.0
}

impl AppConfig {
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            let config: AppConfig =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;

            Ok(config)
        } else {
            // 创建默认配置文件
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// 获取配置文件路径
    fn config_path() -> Result<PathBuf> {
        let config_dir = if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_home)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            return Err(anyhow::anyhow!("Cannot determine config directory"));
        };

        Ok(config_dir.join("rust-study-examples").join("config.toml"))
    }

    /// 获取字体族配置，按优先级排序
    pub fn get_font_families(&self) -> Vec<String> {
        let mut families = Vec::new();

        // 添加中文字体
        families.extend(self.gui.font_families.chinese.clone());
        // 添加英文字体
        families.extend(self.gui.font_families.english.clone());
        // 添加fallback字体
        families.extend(self.gui.font_families.fallback.clone());

        families
    }
}

/// 展开波浪号路径
#[allow(dead_code)]
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}
