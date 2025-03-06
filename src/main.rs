use anyhow::{Context, Result};
use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    os::fd::AsFd,
    path::PathBuf,
};
use zbus::{
    zvariant::{Fd, ObjectPath, OwnedObjectPath, Value},
    Connection, Proxy,
};

const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";
const UDISKS2_MANAGER_PATH: &str = "/org/freedesktop/UDisks2/Manager";
const UDISKS2_MANAGER_INTERFACE: &str = "org.freedesktop.UDisks2.Manager";
const UDISKS2_FILESYSTEM_INTERFACE: &str = "org.freedesktop.UDisks2.Filesystem";
const UDISKS2_LOOP_INTERFACE: &str = "org.freedesktop.UDisks2.Loop";

#[derive(Parser, Debug)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version,
    about = "A tool to test UDisks2 mount interface"
)]
struct Args {
    /// ISO file path
    #[arg(short, long, value_name = "FILE")]
    iso_path: PathBuf,
}

/// Represents a UDisks2 device manager
struct UDisks2Manager<'a> {
    connection: &'a Connection,
    proxy: Proxy<'a>,
}

/// Represents a UDisks2 filesystem device
struct UDisks2Filesystem<'a> {
    proxy: Proxy<'a>,
    object_path: ObjectPath<'static>,
}

impl<'a> UDisks2Manager<'a> {
    async fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            UDISKS2_SERVICE,
            UDISKS2_MANAGER_PATH,
            UDISKS2_MANAGER_INTERFACE,
        )
        .await
        .context("Failed to create UDisks2 manager proxy")?;

        Ok(Self { connection, proxy })
    }

    async fn setup_loop_device(&self, iso_fd: Fd<'_>) -> Result<UDisks2Filesystem<'a>> {
        let options = HashMap::<String, Value>::new();
        let object_path: OwnedObjectPath = self
            .proxy
            .call_method("LoopSetup", &(iso_fd, options))
            .await?
            .body()
            .deserialize()
            .context("Failed to deserialize loop device object path")?;

        println!("Loop device created: {}", object_path);

        UDisks2Filesystem::new(self.connection, object_path.into()).await
    }
}

impl<'a> UDisks2Filesystem<'a> {
    async fn new(connection: &'a Connection, object_path: ObjectPath<'static>) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            UDISKS2_SERVICE,
            object_path.clone(),
            UDISKS2_FILESYSTEM_INTERFACE,
        )
        .await
        .context("Failed to create filesystem proxy")?;

        Ok(Self { proxy, object_path })
    }

    async fn mount(&self) -> Result<String> {
        let mount_options = HashMap::<String, Value>::new();
        let mount_path: String = self
            .proxy
            .call_method("Mount", &(mount_options,))
            .await?
            .body()
            .deserialize()
            .context("Failed to deserialize mount path")?;

        println!("Mounted at: {}", mount_path);
        Ok(mount_path)
    }

    async fn verify_mount_point(&self) -> Result<()> {
        let mount_points: Vec<Vec<u8>> = self
            .proxy
            .get_property("MountPoints")
            .await
            .context("Failed to get mount points")?;

        let mount_point = mount_points
            .first()
            .context("No mount points available")?;

        let mount_point_str = String::from_utf8_lossy(mount_point);
        println!("Actual mount point: {}", mount_point_str);
        Ok(())
    }

    async fn unmount(&self) -> Result<()> {
        let unmount_options = HashMap::<String, Value>::new();
        self.proxy
            .call_method("Unmount", &(unmount_options,))
            .await
            .context("Failed to unmount filesystem")?;

        println!("Unmounted successfully");
        Ok(())
    }

    async fn delete(self) -> Result<()> {
        let loop_proxy = Proxy::new(
            self.proxy.connection(),
            UDISKS2_SERVICE,
            self.object_path.clone(),
            UDISKS2_LOOP_INTERFACE,
        )
        .await
        .context("Failed to create loop device proxy")?;

        let delete_options = HashMap::<String, Value>::new();
        loop_proxy
            .call_method("Delete", &(delete_options,))
            .await
            .context("Failed to delete loop device")?;

        println!("Loop device deleted: {}", self.object_path);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Open the ISO file
    let file = File::open(&args.iso_path)
        .with_context(|| format!("Failed to open ISO file: {}", args.iso_path.display()))?;
    let iso_fd = Fd::from(file.as_fd());
    println!("File descriptor: {}", iso_fd);

    // Connect to the system bus
    let connection = Connection::system()
        .await
        .context("Failed to connect to system bus")?;

    // Create UDisks2 manager and setup loop device
    let manager = UDisks2Manager::new(&connection).await?;
    let filesystem = manager.setup_loop_device(iso_fd).await?;

    // Mount and verify
    filesystem.mount().await?;
    filesystem.verify_mount_point().await?;

    // Cleanup
    filesystem.unmount().await?;
    filesystem.delete().await?;

    Ok(())
}