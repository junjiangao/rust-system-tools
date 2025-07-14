use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use tracing::{error, info};

use zbus::Connection;

mod config;
mod udisks2;
use udisks2::IsoMounter;
use wim_parser::WimParser;

mod gui;
use gui::run_gui;

#[derive(Parser, Debug)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version,
    about = "A tool to test UDisks2 mount interface"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Mount ISO file in console mode
    Mount {
        /// ISO file path
        #[arg(short, long, value_name = "FILE")]
        iso_path: PathBuf,
    },
    /// Show GUI window
    #[command(long_about = "Launch GUI interface for ISO mounting")]
    ShowGui,
}

/// Main application logic
struct App {
    connection: Connection,
}

impl App {
    async fn new() -> Result<Self> {
        let connection = Connection::system()
            .await
            .context("Failed to connect to system bus")?;

        Ok(Self { connection })
    }

    /// 读取并解析挂载目录中的系统信息，优先使用 WIM 解析器
    async fn read_and_parse_system_info(&self, mount_path: &str) -> Result<(String, String)> {
        use std::path::Path;

        info!("开始分析挂载目录中的系统信息");

        // 优先尝试解析 WIM 文件
        let wim_paths = vec![
            format!("{}/sources/install.wim", mount_path),
            format!("{}/sources/install.esd", mount_path),
        ];

        for wim_path in wim_paths {
            if Path::new(&wim_path).exists() {
                info!("找到 WIM 文件: {}", wim_path);

                match self.parse_wim_file(&wim_path).await {
                    Ok((version, arch)) => {
                        info!("成功从 WIM 文件解析版本信息");
                        return Ok((version, arch));
                    }
                    Err(e) => {
                        info!("WIM 文件解析失败，继续尝试其他方法: {}", e);
                    }
                }
            }
        }

        // 如果 WIM 解析失败，回退到原有的文本文件分析方法
        info!("WIM 解析失败，回退到文本文件分析方法");
        self.parse_text_based_system_info(mount_path).await
    }

    /// 解析 WIM 文件获取版本信息
    async fn parse_wim_file(&self, wim_path: &str) -> Result<(String, String)> {
        let mut parser = WimParser::new(wim_path)?;

        // 完整解析 WIM 文件
        parser.parse_full().context("解析 WIM 文件失败")?;

        // 获取 Windows 版本信息
        if let Some(windows_info) = parser.get_windows_info() {
            info!("检测到 Windows 镜像: {}", windows_info);

            // 显示所有镜像的详细信息
            for image in parser.get_images() {
                info!("  {}", image);
            }

            return Ok((windows_info.version, windows_info.architecture));
        }

        // 如果不是 Windows 镜像，尝试从第一个镜像获取信息
        if let Some(first_image) = parser.get_images().first() {
            let version = first_image
                .version
                .clone()
                .unwrap_or_else(|| first_image.name.clone());

            let arch = first_image
                .architecture
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());

            info!("从第一个镜像获取版本信息: {} [{}]", version, arch);
            return Ok((version, arch));
        }

        Err(anyhow::anyhow!("WIM 文件中没有找到有效的镜像信息"))
    }

    /// 基于文本文件的系统信息解析（原有方法）
    async fn parse_text_based_system_info(&self, mount_path: &str) -> Result<(String, String)> {
        use std::fs;
        use std::path::Path;

        // 尝试读取可能的信息来源
        let info_sources = vec![
            format!("{}/sources/idwbinfo.txt", mount_path),
            format!("{}/sources/lang.ini", mount_path),
            format!("{}/README.TXT", mount_path),
            format!("{}/sources/ei.cfg", mount_path),
        ];

        for source_path in info_sources {
            if Path::new(&source_path).exists() {
                match fs::read_to_string(&source_path) {
                    Ok(content) => {
                        info!("读取到信息文件: {}", source_path);
                        return self.parse_system_info_from_text(&content);
                    }
                    Err(e) => {
                        info!("无法读取文件 {}: {}", source_path, e);
                    }
                }
            }
        }

        // 如果没有找到信息文件，尝试基于目录结构推断
        self.infer_system_info_from_structure(mount_path).await
    }

    /// 从文本内容中解析系统信息
    fn parse_system_info_from_text(&self, content: &str) -> Result<(String, String)> {
        let mut version = String::from("Unknown");
        let mut arch = String::from("Unknown");

        // 查找版本信息
        if content.contains("Windows 11") {
            version = "Windows 11".to_string();
        } else if content.contains("Windows 10") {
            version = "Windows 10".to_string();
        } else if content.contains("Windows Server") {
            version = "Windows Server".to_string();
        }

        // 查找架构信息
        if content.contains("x64") || content.contains("amd64") {
            arch = "x64".to_string();
        } else if content.contains("x86") {
            arch = "x86".to_string();
        } else if content.contains("arm64") {
            arch = "ARM64".to_string();
        }

        Ok((version, arch))
    }

    /// 基于目录结构推断系统信息
    async fn infer_system_info_from_structure(&self, mount_path: &str) -> Result<(String, String)> {
        use std::fs;
        use std::path::Path;

        let sources_path = format!("{mount_path}/sources");
        let _system_32_path = format!("{mount_path}/sources/sxs");

        let mut version = String::from("Windows ISO");
        let mut arch = String::from("Unknown");

        // 检查是否存在典型的 Windows ISO 结构
        if Path::new(&sources_path).exists() {
            if let Ok(entries) = fs::read_dir(&sources_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if file_name_str.contains("install.wim")
                        || file_name_str.contains("install.esd")
                    {
                        info!("检测到 Windows 安装文件: {}", file_name_str);
                        version = "Windows".to_string();

                        // 尝试通过文件大小推断架构（这是一个粗略的方法）
                        if let Ok(metadata) = entry.metadata() {
                            let size_gb = metadata.len() / (1024 * 1024 * 1024);
                            if size_gb > 4 {
                                arch = "x64".to_string();
                            } else {
                                arch = "x86".to_string();
                            }
                        }
                        break;
                    }
                }
            }
        }

        Ok((version, arch))
    }

    async fn run(&self, args: Args) -> Result<()> {
        match args.command {
            Commands::Mount { iso_path } => {
                info!("开始控制台挂载 ISO: {:?}", iso_path);
                self.run_console_mode(&iso_path).await?;
                info!("完成控制台挂载 ISO: {:?}", iso_path);
            }
            Commands::ShowGui => {
                info!("启动 GUI 界面");
                self.run_with_gui().await?;
                info!("退出 GUI 界面");
            }
        }
        Ok(())
    }

    async fn run_console_mode(&self, iso_path: &Path) -> Result<()> {
        match self.mount_iso_workflow(iso_path).await {
            Ok(_) => info!("ISO mount workflow completed successfully"),
            Err(e) => {
                error!("Error during ISO mount workflow: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    async fn run_with_gui(&self) -> Result<()> {
        info!("Starting GUI mode...");
        run_gui()
    }

    async fn mount_iso_workflow(&self, iso_path: &Path) -> Result<()> {
        let mounter = IsoMounter::new(&self.connection).await?;
        let mounted_iso = mounter.mount_iso(iso_path).await?;

        info!("ISO successfully mounted at: {}", mounted_iso.mount_path);

        // 读取并打印系统信息
        match self
            .read_and_parse_system_info(&mounted_iso.mount_path)
            .await
        {
            Ok((version, arch)) => {
                info!("系统版本: {}", version);
                info!("系统架构: {}", arch);
            }
            Err(e) => {
                error!("读取系统信息失败: {}", e);
            }
        }

        self.process_mounted_files(&mounted_iso.mount_path).await?;

        mounter.unmount_iso(mounted_iso).await?;
        Ok(())
    }

    /// Example extension method: process the mounted files
    async fn process_mounted_files(&self, mount_path: &str) -> Result<()> {
        info!("Processing files in mount point: {}", mount_path);
        // Insert real file processing logic here
        // For example:
        // - Scan file list
        // - Extract specific files
        // - Analyze ISO contents
        // - Data processing, etc.

        info!("File processing completed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing 日志系统，设置合适的日志级别和格式
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false) // 不显示模块路径，简化输出
        .with_level(true) // 显示日志级别
        .with_thread_ids(false) // 不显示线程ID，简化输出
        .init();

    let args = Args::parse();
    let app = App::new().await?;

    if let Err(e) = app.run(args).await {
        error!("Application error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
