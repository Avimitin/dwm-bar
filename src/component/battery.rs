use super::widget::Block;
use tokio::fs;

/// Build a component to show laptop battery percentage and power-supply status.
/// Statistic come from /sys/class/power_supply/<bat_name>/{capacity,status}.
///
/// Return None if no battery device name contains "BAT0" keyword,
/// or no capacity/status file was found.
pub async fn battery() -> Option<Block> {
    let perc = tokio::spawn(async {
        fs::read_to_string(format!("/sys/class/power_supply/{}/capacity", "BAT0"))
                .await
                .ok()?
                .parse::<i32>()
                .ok()
    });

    let stat = fs::read_to_string(format!("/sys/class/power_supply/{}/status", "BAT0"))
        .await
        .ok()?;

    let icon = if stat == "Discharging" { "" } else { "" };

    let perc = perc.await.unwrap()?;

    Some(
        Block::new(icon, format!("{} %", perc))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}
