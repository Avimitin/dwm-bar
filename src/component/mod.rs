use std::process::*;
macro_rules! cmd {
    ($c:expr, $($a:expr),*) => {
        {
            let mut args = vec![$c];
            $(
                args.push($a);
            )*
            let raw = Command::new("sh")
                .arg("-c")
                .arg(args.join(" "))
                .output()
                .expect(format!("Fail to execute {} command", $c).as_str());
            let stdout = String::from_utf8(raw.stdout);
            match stdout {
                Ok(s) => s.trim().to_owned(),
                Err(e) => panic!("Unreadable output from command {} {:?}: {}", $c, &args, e),
            }
        }
    };
    ($c:expr) => {
        {
            let raw = Command::new("sh")
                .arg("-c")
                .arg($c)
                .output()
                .expect(format!("Fail to execute {} command", $c).as_str());
            let stdout = String::from_utf8(raw.stdout);
            match stdout {
                Ok(s) => s.trim().to_owned(),
                Err(e) => panic!("Unreadable output from command {}: {}", $c, e),
            }
        }
    }
}

mod color;

use color::Color;

#[derive(Debug)]
pub struct Component {
    color: Color,
    text: String,
    icon: String,
}

impl Component {
    /// Builder chain for component.
    pub fn new<T: Into<String>, E: Into<String>>(icon: T, text: E) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
            color: Color::new(),
        }
    }

    pub fn text_fg<T: Into<String>>(mut self, fg: T) -> Self {
        self.color = self.color.text_fg(fg);
        self
    }

    pub fn text_color<T: Into<String>>(mut self, fg: T, bg: T) -> Self {
        self.color = self.color.text(fg, bg);
        self
    }

    pub fn icon_color<T: Into<String>>(mut self, fg: T, bg: T) -> Self {
        self.color = self.color.icon(fg, bg);
        self
    }

    pub fn icon_fg<T: Into<String>>(mut self, fg: T) -> Self {
        self.color = self.color.icon_fg(fg);
        self
    }
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::new();

        let fg = self.color.fg.as_ref().unwrap_or(&s);
        let bg = self.color.bg.as_ref().unwrap_or(&s);
        let icon_fg = self.color.icon_fg.as_ref().unwrap_or(&s);
        let icon_bg = self.color.icon_bg.as_ref().unwrap_or(&s);
        // [icon] [text]
        write!(
            f,
            "{}{}{} {}{}{}",
            icon_fg, icon_bg, self.icon, fg, bg, self.text
        )
    }
}

/// Create a date component for bar
pub fn date_and_time() -> Option<Component> {
    use chrono::prelude::Local;
    let now = Local::now();
    Some(
        Component::new("", now.format("%B/%d %I:%M %p").to_string())
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}

/// Create a sound volume component for bar
pub fn sound_volume() -> Option<Component> {
    // TODO: use the libpulse crates to do this shit
    let output = cmd!("pamixer", "--get-volume");
    Some(
        Component::new("", format!("{}%", output))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}

pub fn song_info() -> Option<Component> {
    use mpris::PlayerFinder;
    // TODO: We need to use logging to report error here.
    let player = PlayerFinder::new().ok()?.find_active().ok()?;

    let text_limit = 40;
    let metadata = player.get_metadata().ok()?;

    let artist = metadata.artists()?.join(" ");
    let song = metadata.title()?;

    let output = format!(
        " {} - {} ",
        if !artist.is_empty() {
            artist
        } else {
            "Anonymous".to_string()
        },
        song,
    );

    // trim the text
    let output = if output.len() > text_limit {
        format!("{}...", &output[0..text_limit])
    } else {
        output
    };

    Some(
        Component::new(" ", output)
            .icon_color("#EAEAEA", "#0C0C0C")
            .text_color("#EAEAEA", "#171617"),
    )
}

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

pub fn avg_load() -> Option<Component> {
    use std::fs;

    let status = fs::read_to_string("/proc/stat").ok()?;
    let mut cpustat = Vec::new();
    for line in status.lines() {
        if line.starts_with("cpu") {
            cpustat = line.split(' ').skip(2).collect::<Vec<&str>>();
            break;
        }
    }

    if cpustat.len() < 8 {
        return None;
    }

    // get the cpu idle time
    let idle = cpustat.remove(3).parse::<f32>().ok()?;
    let mut other = 0.0;
    for time in cpustat {
        other += time.parse::<f32>().ok()?;
    }

    let avg = other / idle;

    Some(
        Component::new("﬙", format!("{:.2} %", avg * 100.0))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}
