use std::process::Command;
use super::component::Component;

pub fn battery() -> Option<Component> {
    let output = cmd!("acpi");
    let output: Vec<&str> = output.split(' ').collect();
    if output.is_empty() {
        return None;
    }

    let icon = if output[2] == "Discharging," {
        ""
    } else {
        ""
    };

    Some(
        Component::new(icon, output[3])
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}

pub fn headset_battery() -> Option<Component> {
    let headset = cmd!("upower", "-e");
    if headset.is_empty() {
        return None;
    }

    let mut device: &str = "";
    for line in headset.lines() {
        if line.contains("headset") {
            device = line;
        }
    }

    if device.is_empty() {
        return None;
    }

    let info = cmd!("upower", "-i", device);
    if info.is_empty() {
        return None;
    }

    let mut battery = "";
    for line in info.lines() {
        if line.contains("percentage") {
            battery = line;
        }
    }
    if battery.is_empty() {
        return None;
    }

    let percentage: Vec<&str> = battery.matches(char::is_numeric).collect();
    if percentage.is_empty() {
        None
    } else {
        Some(
            Component::new("", format!("{}%", percentage.join("")))
                .text_fg("#EAEAEA")
                .icon_fg("#EAEAEA"),
        )
    }
}

