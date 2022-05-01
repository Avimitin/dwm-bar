use super::widget::Block;
use anyhow::Result;
use async_trait::async_trait;
use dbus::arg;
use dbus::nonblock::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection};
use dbus_tokio::connection;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct SongInfo {
    conn: Arc<SyncConnection>,
}

impl SongInfo {
    pub fn new() -> Result<SongInfo> {
        let (resource, conn) = connection::new_session_sync()?;
        let _conn_handle = tokio::spawn(async {
            info!("Holding connection to D-Bus");
            let err = resource.await;
            error!("Lost connection to D-Bus: {}", err);
        });
        Ok(SongInfo { conn })
    }

    async fn get_metadata(&self) -> Result<arg::PropMap> {
        let proxy = Proxy::new(
            "org.freedesktop.DBus",
            "/",
            Duration::from_millis(2000),
            self.conn.clone(),
        );
        let (services,): (Vec<String>,) = proxy
            .method_call("org.freedesktop.DBus", "ListNames", ())
            .await?;

        let addr = services
            .iter()
            .find(|serv| serv.contains("mpris"))
            .ok_or_else(|| anyhow::anyhow!("No mpris device found"))?;

        let proxy = Proxy::new(
            addr,
            "/org/mpris/MediaPlayer2",
            Duration::from_millis(2000),
            self.conn.clone(),
        );
        Ok(proxy
            .get("org.mpris.MediaPlayer2.Player", "Metadata")
            .await?)
    }
}

#[async_trait]
impl super::Updater for SongInfo {
    async fn update(&mut self) -> Option<Block> {
        let metadata = self.get_metadata().await.ok()?;

        let artist: Option<&Vec<String>> = arg::prop_cast(&metadata, "xesam:artist");
        let artist = artist?.join(" ");
        let song: Option<&String> = arg::prop_cast(&metadata, "xesam:title");
        let song = song?;

        let output = format!(
            " {} - {} ",
            if !artist.is_empty() {
                artist
            } else {
                "Anonymous".to_string()
            },
            song,
        );

        let text_limit = 40;
        // trim the text
        let output = if output.len() > text_limit {
            let split = output.chars().take(text_limit).collect::<String>();
            format!("{}...", split)
        } else {
            output
        };

        Some(
            Block::new(" ", output)
                .icon_color("#EAEAEA", "#0C0C0C")
                .text_color("#EAEAEA", "#171617"),
        )
    }
}
