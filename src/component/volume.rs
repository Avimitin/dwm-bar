use super::widget::Block;
use tokio::process::Command;

/// Create a sound volume component for bar
pub async fn sound_volume() -> Option<Block> {
    // TODO: use the libpulse crates to do this shit
    let output = Command::new("pamixer")
        .arg("--get-volume")
        .output();

    let output = output.await.ok()?;

    if !output.status.success() {
        return None;
    }

    let output = String::from_utf8(output.stdout).ok()?;
    Some(
        Block::new("", format!("{}%", output.trim()))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}
