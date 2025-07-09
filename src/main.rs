use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use tracing::{error, info};

use zbus::Connection;

mod config;
mod udisks2;
use udisks2::IsoMounter;

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

    /// 读取并解析挂载目录中的source/install.wim头部，提取系统版本和架构信息
    async fn read_and_parse_wim_header(&self, mount_path: &str) -> Result<(String, String)> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        use quick_xml::name::QName;
        use std::fs::File;
        use std::io::Read as _;

        let wim_path = format!("{mount_path}/sources/install.wim");
        let mut file = File::open(&wim_path).context("无法打开install.wim文件")?;

        let mut buffer = vec![0u8; 4096];
        file.read_exact(&mut buffer)
            .context("读取install.wim文件头失败")?;

        let mut reader = Reader::from_reader(&buffer[..]);
        reader.trim_text(true);

        let mut version = String::new();
        let mut arch = String::new();

        let mut in_image = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if e.name() == QName(b"IMAGE") {
                        in_image = true;
                    } else if in_image {
                        if e.name() == QName(b"ARCH") {
                            arch = reader
                                .read_text(QName(b"ARCH"))
                                .unwrap_or_default()
                                .to_string();
                        } else if e.name() == QName(b"NAME") {
                            version = reader
                                .read_text(QName(b"NAME"))
                                .unwrap_or_default()
                                .to_string();
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name() == QName(b"IMAGE") {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow::anyhow!("XML 解析失败: {}", e));
                }
                _ => {}
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

        // 读取并打印install.wim文件头信息
        match self
            .read_and_parse_wim_header(&mounted_iso.mount_path)
            .await
        {
            Ok((version, arch)) => {
                info!("WIM 文件版本: {}", version);
                info!("WIM 文件架构: {}", arch);
            }
            Err(e) => {
                error!("读取WIM文件头失败: {}", e);
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
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let app = App::new().await?;

    if let Err(e) = app.run(args).await {
        error!("Application error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
