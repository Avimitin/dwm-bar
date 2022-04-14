use super::widget::Block;
use anyhow::Result;
use dbus::arg;
use dbus::blocking::{Connection, stdintf::org_freedesktop_dbus::Properties};
use std::time::Duration;

pub async fn song_info() -> Option<Block> {
    let conn = Connection::new_session().ok()?;
    let player_addr = find_active_player_address(&conn).ok()?;
    let metadata = get_metadata(&conn, &player_addr).ok()?;

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

fn find_active_player_address(conn: &Connection) -> Result<String> {
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(2000));
    let (services,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;

    for service in services {
        if service.contains("mpris") {
            return Ok(service);
        }
    }

    anyhow::bail!("No active mpris player was found")
}

fn get_metadata(conn: &Connection, addr: &str) -> Result<arg::PropMap> {
    let proxy = conn.with_proxy(addr, "/org/mpris/MediaPlayer2", Duration::from_millis(2000));
    Ok(proxy.get("org.mpris.MediaPlayer2.Player", "Metadata")?)
}
