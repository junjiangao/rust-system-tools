use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    os::fd::AsFd,
    path::{Path, PathBuf},
};
#[allow(unused_imports)]
use tracing::{debug, info, warn};
use zbus::{
    Connection, Proxy,
    zvariant::{Fd, ObjectPath, OwnedObjectPath, Value},
};

const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";
const UDISKS2_MANAGER_PATH: &str = "/org/freedesktop/UDisks2/Manager";
const UDISKS2_MANAGER_INTERFACE: &str = "org.freedesktop.UDisks2.Manager";
const UDISKS2_FILESYSTEM_INTERFACE: &str = "org.freedesktop.UDisks2.Filesystem";
const UDISKS2_LOOP_INTERFACE: &str = "org.freedesktop.UDisks2.Loop";

/// Represents a UDisks2 device manager
pub struct UDisks2Manager<'a> {
    connection: &'a Connection,
    proxy: Proxy<'a>,
}

/// Represents a UDisks2 filesystem device
pub struct UDisks2Filesystem<'a> {
    proxy: Proxy<'a>,
    object_path: ObjectPath<'static>,
}

/// High-level ISO mounting manager
pub struct IsoMounter<'a> {
    manager: UDisks2Manager<'a>,
}

/// Represents a mounted ISO with its metadata
pub struct MountedIso<'a> {
    pub mount_path: String,
    pub filesystem: UDisks2Filesystem<'a>,
    pub iso_path: PathBuf,
}

impl<'a> UDisks2Manager<'a> {
    pub async fn new(connection: &'a Connection) -> Result<Self> {
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

    pub async fn setup_loop_device(&self, iso_fd: Fd<'_>) -> Result<UDisks2Filesystem<'a>> {
        let options = HashMap::<String, Value>::new();
        let object_path: OwnedObjectPath = self
            .proxy
            .call_method("LoopSetup", &(iso_fd, options))
            .await?
            .body()
            .deserialize()
            .context("Failed to deserialize loop device object path")?;

        info!("Loop device created: {object_path}");

        UDisks2Filesystem::new(self.connection, object_path.into()).await
    }
}

impl<'a> UDisks2Filesystem<'a> {
    pub async fn new(connection: &'a Connection, object_path: ObjectPath<'static>) -> Result<Self> {
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

    pub async fn mount(&self) -> Result<String> {
        let mount_options = HashMap::<String, Value>::new();
        let mount_path: String = self
            .proxy
            .call_method("Mount", &(mount_options,))
            .await?
            .body()
            .deserialize()
            .context("Failed to deserialize mount path")?;

        info!("Mounted at: {mount_path}");
        Ok(mount_path)
    }

    pub async fn verify_mount_point(&self) -> Result<()> {
        let mount_points: Vec<Vec<u8>> = self
            .proxy
            .get_property("MountPoints")
            .await
            .context("Failed to get mount points")?;

        let mount_point = mount_points.first().context("No mount points available")?;

        let mount_point_str = String::from_utf8_lossy(mount_point);
        debug!("Actual mount point: {mount_point_str}");
        Ok(())
    }

    pub async fn unmount(&self) -> Result<()> {
        let unmount_options = HashMap::<String, Value>::new();
        self.proxy
            .call_method("Unmount", &(unmount_options,))
            .await
            .context("Failed to unmount filesystem")?;

        info!("Unmounted successfully");
        Ok(())
    }

    pub async fn delete(self) -> Result<()> {
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

        info!("Loop device deleted: {}", self.object_path);
        Ok(())
    }
}

impl<'a> IsoMounter<'a> {
    pub async fn new(connection: &'a Connection) -> Result<Self> {
        let manager = UDisks2Manager::new(connection).await?;
        Ok(Self { manager })
    }

    /// Mount an ISO file and return the mount path and filesystem handler
    pub async fn mount_iso<P: AsRef<Path>>(&self, iso_path: P) -> Result<MountedIso<'a>> {
        let path = iso_path.as_ref();
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "ISO file does not exist: {}",
                path.display()
            ));
        }

        // Open the ISO file
        let file = File::open(path)
            .with_context(|| format!("Failed to open ISO file: {}", path.display()))?;
        let iso_fd = Fd::from(file.as_fd());
        debug!("Opening ISO file: {} (fd: {})", path.display(), iso_fd);

        // Setup loop device and mount
        let filesystem = self.manager.setup_loop_device(iso_fd).await?;
        let mount_path = filesystem.mount().await?;
        filesystem.verify_mount_point().await?;

        Ok(MountedIso {
            mount_path,
            filesystem,
            iso_path: path.to_path_buf(),
        })
    }

    /// Unmount and cleanup an ISO filesystem
    pub async fn unmount_iso(&self, mounted_iso: MountedIso<'a>) -> Result<()> {
        info!("Unmounting ISO: {}", mounted_iso.iso_path.display());
        mounted_iso.filesystem.unmount().await?;
        mounted_iso.filesystem.delete().await?;
        Ok(())
    }
}
