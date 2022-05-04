use super::widget::Block;
use anyhow::{Context, Result};
use async_trait::async_trait;
use dbus::nonblock::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection};
use dbus::Path;
use dbus_tokio::connection;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

/// Build a headset battery component.
/// This functionality depends on UPower DBus daemon
///
/// Return None if no battery device name contains "headset" keyword,
/// or no percentage property is found.
pub struct HeadsetBattery<'a> {
    proxy: Option<Proxy<'a, Arc<SyncConnection>>>,
    conn: Arc<SyncConnection>,
    close_notifier: tokio::sync::oneshot::Sender<u8>,
}

#[async_trait]
impl<'a> super::Updater for HeadsetBattery<'a> {
    /// Update headset battery information. It will automatically update devices when dbus
    /// connection is failed.
    async fn update(&mut self) -> Option<Block> {
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
}

impl<'a> Drop for HeadsetBattery<'_> {
    fn drop(&mut self) {
        self.close_notifier.send(0);
    }
}

impl<'a> HeadsetBattery<'a> {
    pub async fn new() -> Result<HeadsetBattery<'a>> {
        let (resource, conn) = connection::new_system_sync()?;

        // notify the resource holder to close the connection
        let (tx, rx) = oneshot::channel::<u8>();

        // hold the connection in other thread
        tokio::spawn(async move {
            tracing::trace!("Holding DBus connectoin to headset battery");
            tokio::select!{
                // close connection when headset battery instant is dropped
                _ = rx => {
                    tracing::trace!("Headset Battery: UPower DBus Connection closed by signal");
                    return;
                }
                // if the resources can't be await, it means something unexpected happened
                err = resource => {
                    tracing::trace!("Headset Battery: UPower DBus Connection closed unexpectedly: {}", err);
                    return;
                }
            }
        });

        let mut bat = Self { proxy: None, close_notifier: tx , conn };

        // try get device at initialize
        bat.enum_devices().await?;

        Ok(bat)
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
