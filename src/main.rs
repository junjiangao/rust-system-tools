use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::os::fd::AsFd;
use std::path::PathBuf;
use zbus::zvariant::{Fd, ObjectPath, OwnedObjectPath, Value};
use zbus::{Connection, Proxy};

const MANAGER_PATH: &str = "/org/freedesktop/UDisks2/Manager";
const MANAGER_INTERFACE: &str = "org.freedesktop.UDisks2.Manager";

#[derive(Parser, Debug)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version,
    about = "A tool to test udisk2 mount interface",
)]
struct Args {
    /// ISO file path
    #[arg(short, long)]
    iso_path: PathBuf,
}

async fn create_proxy<'a, P: Into<ObjectPath<'a>>>(
    connection: &Connection,
    path: P,
    interface: &'a str,
) -> Result<Proxy<'a>> {
    Proxy::new(connection, "org.freedesktop.UDisks2", path, interface)
        .await
        .context("Failed to create proxy")
}

async fn setup_loop_device(connection: &Connection, iso_fd: Fd<'_>) -> Result<ObjectPath<'static>> {
    let manager_proxy = create_proxy(connection, OwnedObjectPath::try_from(MANAGER_PATH)?, MANAGER_INTERFACE).await?;
    let options = HashMap::<String, Value>::new();
    let ret = manager_proxy
        .call_method("LoopSetup", &(iso_fd, options))
        .await?
        .body()
        .to_owned();
    let object_path: OwnedObjectPath = ret.deserialize()?;
    println!("Loop device created: {}", object_path);
    Ok(object_path.into())
}

async fn mount_filesystem(
    connection: &Connection,
    object_path: &ObjectPath<'static>,
) -> Result<String> {
    let fs_proxy = create_proxy(
        connection,
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
    println!("Mounted at: {}", mount_path);
    Ok(mount_path)
}

async fn verify_mount_point(fs_proxy: &Proxy<'_>) -> Result<()> {
    let mount_points: Vec<Vec<u8>> = fs_proxy.get_property("MountPoints").await?;
    let mount_point = mount_points.first().context("Missing mount point")?;
    println!(
        "Actual mount point: {}",
        std::str::from_utf8(mount_point).unwrap_or_default()
    );
    Ok(())
}

async fn unmount_filesystem(fs_proxy: &Proxy<'_>) -> Result<()> {
    let unmount_options = HashMap::<String, Value>::new();
    fs_proxy.call_method("Unmount", &(unmount_options)).await?;
    println!("Unmounted successfully");
    Ok(())
}

async fn delete_loop_device(
    connection: &Connection,
    object_path: &ObjectPath<'static>,
) -> Result<()> {
    let loop_proxy = create_proxy(
        connection,
        object_path.clone(),
        "org.freedesktop.UDisks2.Loop",
    )
    .await?;
    let delete_options = HashMap::<String, Value>::new();
    loop_proxy.call_method("Delete", &(delete_options)).await?;
    println!("Loop device deleted: {}", object_path);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file = File::open(&args.iso_path).context("Unable to open file")?;
    let iso_fd = Fd::from(file.as_fd());
    println!("File descriptor: {}", iso_fd);

    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    let object_path = setup_loop_device(&connection, iso_fd).await?;

    mount_filesystem(&connection, &object_path).await?;

    let fs_proxy = create_proxy(
        &connection,
        object_path.clone(),
        "org.freedesktop.UDisks2.Filesystem",
    )
    .await?;
    verify_mount_point(&fs_proxy).await?;

    unmount_filesystem(&fs_proxy).await?;
    delete_loop_device(&connection, &object_path).await?;

    Ok(())
}
