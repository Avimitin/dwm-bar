use super::widget::Block;
use anyhow::{Context, Result};
use dbus::nonblock::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection};
use dbus::Path;
use dbus_tokio::connection;
use std::sync::Arc;
use std::time::Duration;

/// Build a headset battery component.
/// This functionality depends on UPower DBus daemon
///
/// Return None if no battery device name contains "headset" keyword,
/// or no percentage property is found.
pub struct HeadsetBattery<'a> {
    proxy: Option<Proxy<'a, Arc<SyncConnection>>>,
    conn: Arc<SyncConnection>,
}

impl<'a> HeadsetBattery<'a> {
    pub async fn new() -> Result<HeadsetBattery<'a>> {
        let (resource, conn) = connection::new_system_sync()?;
        // hold the connection in other thread
        tokio::spawn(async {
            resource.await;
        });

        let mut bat = Self { proxy: None, conn };

        // try get device at initialize
        bat.enum_devices().await?;

        Ok(bat)
    }

    pub async fn update(&mut self) -> Option<Block> {
        // check and update headset device
        self.enum_devices().await.ok()?;

        let proxy = self.proxy.as_ref()?;

        let percentage: f64 = proxy
            .get("org.freedesktop.UPower.Device", "Percentage")
            .await
            .map_err(|e| {
                // remove the available device, try update it next enumerate function call.
                self.proxy.take();
                tracing::error!("Fail to get headset battery percentage: {}", e);
            })
            .ok()?;

        Some(
            Block::new("", format!("{:.0}%", percentage))
                .text_fg("#EAEAEA")
                .icon_fg("#EAEAEA"),
        )
    }

    async fn enum_devices(&mut self) -> Result<()> {
        // early return when headset device is already available
        if self.proxy.is_some() {
            return Ok(());
        }

        let proxy = Proxy::new(
            "org.freedesktop.UPower",
            "/org/freedesktop/UPower",
            Duration::from_millis(2000),
            self.conn.clone(),
        );

        let (devices,): (Vec<Path>,) = proxy
            .method_call("org.freedesktop.UPower", "EnumerateDevices", ())
            .await
            .with_context(|| "Fail to enumerate available devices for headset battery")?;

        let device = devices
            .iter()
            .find(|dev| dev.contains("headset"))
            .ok_or_else(|| anyhow::anyhow!("No headset device found"))?;

        self.proxy.replace(Proxy::new(
            "org.freedesktop.UPower",
            device.clone(),
            Duration::from_millis(2000),
            self.conn.clone(),
        ));

        Ok(())
    }
}
