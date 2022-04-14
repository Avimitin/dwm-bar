use super::widget::Block;
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::blocking::Connection;
use dbus::Path;
use std::fs;
use std::time::Duration;

/// Build a component to show laptop battery percentage and power-supply status.
/// Statistic come from /sys/class/power_supply/<bat_name>/{capacity,status}.
///
/// Return None if no battery device name contains "BAT0" keyword,
/// or no capacity/status file was found.
pub fn battery() -> Option<Block> {
    let perc: u32 = fs::read_to_string(format!("/sys/class/power_supply/{}/capacity", "BAT0"))
        .ok()?
        .parse()
        .ok()?;
    let stat = fs::read_to_string(format!("/sys/class/power_supply/{}/status", "BAT0")).ok()?;

    let icon = if stat == "Discharging" { "" } else { "" };

    Some(
        Block::new(icon, format!("{} %", perc))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}

/// Build a headset battery component.
/// This functionality depends on UPower DBus daemon
///
/// Return None if no battery device name contains "headset" keyword,
/// or no percentage property is found.
pub fn headset_battery() -> Option<Block> {
    let dbus_sys_connection = Connection::new_system()
        .expect(
            "Fail to connect to dbus, please ensure you are running in Linux with DBus, or ensure you have DBus daemon enabled."
        );
    let proxy = dbus_sys_connection.with_proxy(
        "org.freedesktop.UPower",
        "/org/freedesktop/UPower",
        Duration::from_millis(2000),
    );

    let (devices,): (Vec<Path>,) = proxy
        .method_call("org.freedesktop.UPower", "EnumerateDevices", ())
        .ok()?;

    let mut device = Path::default();
    for dev in devices {
        if dev.contains("headset") {
            device = dev;
            break;
        }
    }

    if device.is_empty() {
        return None;
    }

    let proxy = dbus_sys_connection.with_proxy(
        "org.freedesktop.UPower",
        device,
        Duration::from_millis(2000),
    );

    let percentage: f64 = proxy
        .get("org.freedesktop.UPower.Device", "Percentage")
        .ok()?;

    Some(
        Block::new("", format!("{:.0}%", percentage))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}
