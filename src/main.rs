use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use zbus::Connection;

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

    async fn run(&self, args: Args) -> Result<()> {
        match args.command {
            Commands::Mount { iso_path } => {
                self.run_console_mode(&iso_path).await?;
            }
            Commands::ShowGui => {
                self.run_with_gui().await?;
            }
        }
        Ok(())
    }

    async fn run_console_mode(&self, iso_path: &PathBuf) -> Result<()> {
        match self.mount_iso_workflow(iso_path).await {
            Ok(_) => println!("ISO mount workflow completed successfully"),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }

        async fn run_with_gui(&self) -> Result<()> {
        println!("Starting GUI mode...");
        run_gui()
    }

    async fn mount_iso_workflow(&self, iso_path: &PathBuf) -> Result<()> {
        let mounter = IsoMounter::new(&self.connection).await?;
        let mounted_iso = mounter.mount_iso(iso_path).await?;

        println!("ISO successfully mounted at: {}", mounted_iso.mount_path);

        // 在这里可以添加其他操作，比如文件处理、数据分析等
        self.process_mounted_files(&mounted_iso.mount_path).await?;

        mounter.unmount_iso(mounted_iso).await?;
        Ok(())
    }

    /// 示例扩展方法：处理挂载的文件
    async fn process_mounted_files(&self, mount_path: &str) -> Result<()> {
        println!("Processing files in mount point: {}", mount_path);
        // 这里可以添加实际的文件处理逻辑
        // 例如：
        // - 扫描文件列表
        // - 提取特定文件
        // - 分析ISO内容
        // - 数据处理等

        println!("File processing completed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let app = App::new().await?;
    app.run(args).await
}
