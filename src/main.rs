use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::os::fd::AsFd;
use std::path::PathBuf;
use zbus::zvariant::{Fd, ObjectPath, OwnedObjectPath, Value};
use zbus::{Connection, Proxy};

const MANAGER_PATH: &str = "/org/freedesktop/UDisks2/Manager";
const MANAGER_INTERFACE: &str = "org.freedesktop.UDisks2.Manager";

// 定义命令行参数结构体
#[derive(Parser, Debug)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version,
    about = "A tool to test udisk2 mount interface",
    long_about = None
)]
struct Args {
    /// ISO 文件路径
    #[arg(short, long)]
    iso_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZBUS error: {0}")]
    Zbus(#[from] zbus::Error),
    #[error("ZVariant error: {0}")]
    ZVariant(#[from] zbus::zvariant::Error),
    #[error("Missing mount point")]
    MissingMountPoint,
}

type Result<T> = std::result::Result<T, Error>;

// 创建代理对象的辅助函数
async fn create_proxy<'a, P: Into<ObjectPath<'a>>>(
    connection: &Connection,
    path: P,
    interface: &'a str,
) -> Result<Proxy<'a>> {
    Ok(Proxy::new(connection, "org.freedesktop.UDisks2", path, interface).await?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 打开文件并获取文件描述符
    let file = File::open(&args.iso_path).map_err(|e| {
        eprintln!("无法打开文件 '{}'", args.iso_path.display());
        e
    })?;
    let iso_fd = Fd::from(file.as_fd());
    println!("文件描述符: {}", iso_fd);

    let connection = Connection::system().await?;

    // 创建 Loop 设备
    let manager_path = OwnedObjectPath::try_from(MANAGER_PATH).map_err(|e| {
        println!("Failed to convert manager path '{}': {}", MANAGER_PATH, e);
        e
    })?;

    let manager_proxy = create_proxy(&connection, manager_path, MANAGER_INTERFACE).await?;

    let options = HashMap::<String, Value>::new();
    let ret = manager_proxy
        .call_method("LoopSetup", &(iso_fd, options))
        .await?
        .body();
    let object_path: ObjectPath = ret.deserialize()?;
    println!("创建 Loop 设备: {}", object_path);

    // 挂载文件系统
    let fs_proxy = create_proxy(
        &connection,
        object_path.clone(),
        "org.freedesktop.UDisks2.Filesystem",
    )
    .await?;

    let mount_options = HashMap::<String, Value>::new();
    let mount_path: String = fs_proxy
        .call_method("Mount", &(mount_options))
        .await?
        .body()
        .deserialize()?;
    println!("成功挂载到: {}", mount_path);

    // 验证挂载点
    let mount_points: Vec<Vec<u8>> = fs_proxy.get_property("MountPoints").await?;
    let mount_point = mount_points.first().ok_or(Error::MissingMountPoint)?;
    println!(
        "实际挂载点: {}",
        std::str::from_utf8(mount_point).unwrap_or_default()
    );

    // 卸载文件系统
    let unmount_options = HashMap::<String, Value>::new();
    fs_proxy.call_method("Unmount", &(unmount_options)).await?;
    println!("成功卸载: {}", mount_path);

    // 删除 Loop 设备
    let loop_proxy = create_proxy(
        &connection,
        object_path.clone(),
        "org.freedesktop.UDisks2.Loop",
    )
    .await?;

    let delete_options = HashMap::<String, Value>::new();
    loop_proxy.call_method("Delete", &(delete_options)).await?;
    println!("已删除 Loop 设备: {}", object_path);

    Ok(())
}
