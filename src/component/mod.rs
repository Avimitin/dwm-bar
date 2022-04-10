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

#[derive(Debug, PartialEq)]
struct Color {
    fg: Option<String>,
    bg: Option<String>,
    icon_fg: Option<String>,
    icon_bg: Option<String>,
}

impl Color {
    /// Create a new color set. The first parameter `fg` means foreground, and the second
    /// parameter `bg` means background. You should give a valid hex color code in format
    /// like `#FFFFFF`. If you pass empty string, this means you are going to use the
    /// default colors.
    fn new<T: Into<String>>(fg: T, bg: T, icon: (T, T)) -> Self {
        let fg = fg.into();
        let bg = bg.into();
        let icon_fg = icon.0.into();
        let icon_bg = icon.1.into();

        let fg = if fg.is_empty() {
            None
        } else {
            Some(format!("^c{}^", fg))
        };

        let bg = if bg.is_empty() {
            None
        } else {
            Some(format!("^b{}^", bg))
        };

        let icon_fg = if icon_fg.is_empty() {
            fg.clone()
        } else {
            Some(format!("^c{}^", icon_fg))
        };

        let icon_bg = if icon_bg.is_empty() {
            bg.clone()
        } else {
            Some(format!("^b{}^", icon_bg))
        };

        Self {
            fg,
            bg,
            icon_fg,
            icon_bg,
        }
    }
}

#[test]
fn test_color_new() {
    assert_eq!(
        Color::new("", "", ("", "")),
        Color {
            fg: None,
            bg: None,
            icon_fg: None,
            icon_bg: None
        }
    );
    assert_eq!(
        Color::new("#000000", "", ("", "")),
        Color {
            fg: Some("^c#000000^".to_string()),
            bg: None,
            icon_fg: Some("^c#000000^".to_string()),
            icon_bg: None,
        }
    );
    assert_eq!(
        Color::new("", "#FFFFFF", ("", "")),
        Color {
            fg: None,
            bg: Some("^b#FFFFFF^".to_string()),
            icon_fg: None,
            icon_bg: Some("^b#FFFFFF^".to_string())
        }
    );
    assert_eq!(
        Color::new("#000000", "#FFFFFF", ("", "")),
        Color {
            fg: Some("^c#000000^".to_string()),
            bg: Some("^b#FFFFFF^".to_string()),
            icon_fg: Some("^c#000000^".to_string()),
            icon_bg: Some("^b#FFFFFF^".to_string()),
        }
    );
    assert_eq!(
        Color::new("#000000", "#FFFFFF", ("#EAEAEA", "#FF00FF")),
        Color {
            fg: Some("^c#000000^".to_string()),
            bg: Some("^b#FFFFFF^".to_string()),
            icon_fg: Some("^c#EAEAEA^".to_string()),
            icon_bg: Some("^b#FF00FF^".to_string()),
        }
    );
}

#[derive(Debug)]
pub struct Component {
    color: Color,
    text: String,
    icon: String,
}

impl Component {
    /// Create a new component with icon, text, and foreground, backgroun colors.
    ///
    /// T: Into<String>
    pub fn new<T: Into<String>>(icon: T, icon_color: (T, T), text: T, text_color: (T, T)) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
            color: Color::new(text_color.0, text_color.1, icon_color),
        }
    }
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fg = String::new();
        let mut bg = String::new();
        let mut icon_fg = String::new();
        let mut icon_bg = String::new();
        if let Some(s) = &self.color.fg {
            fg = s.clone();
        }
        if let Some(s) = &self.color.bg {
            bg = s.clone();
        }
        if let Some(s) = &self.color.icon_fg {
            icon_fg = s.clone();
        }
        if let Some(s) = &self.color.icon_bg {
            icon_bg = s.clone();
        }
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
    Some(Component::new(
        "",
        ("", ""),
        &now.format("%B/%d %I:%M %p").to_string(),
        ("#EAEAEA", ""),
    ))
}

/// Create a sound volume component for bar
pub fn sound_volume() -> Option<Component> {
    // TODO: use the libpulse crates to do this shit
    let output = cmd!("pamixer", "--get-volume");
    Some(Component::new(
        "",
        ("", ""),
        format!("{}%", output).as_str(),
        ("#EAEAEA", ""),
    ))
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

    Some(Component::new(
        " ",
        ("#EAEAEA", "#0C0C0C"),
        &output,
        ("#EAEAEA", "#171617"),
    ))
}

pub fn battery() -> Option<Component> {
    let output = cmd!("acpi");
    let output: Vec<&str> = output.split(' ').collect();
    if output.is_empty() {
        return None;
    }
    if output[2] == "Discharging," {
        Some(Component::new("", ("", ""), output[3], ("#EAEAEA", "")))
    } else {
        Some(Component::new("", ("", ""), output[3], ("#EAEAEA", "")))
    }
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
        Some(Component::new(
            "",
            ("", ""),
            &format!("{}%", percentage.join("")),
            ("#EAEAEA", ""),
        ))
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
    let idle = cpustat.remove(3).parse::<i32>().ok()?;
    let mut other = 0;
    for time in cpustat {
        other += time.parse::<i32>().ok()?;
    }

    let avg = other / idle;

    Some(Component::new(
        "﬙",
        ("", ""),
        &format!("{} %", avg / 100), // use the load from last minutes
        ("#EAEAEA", ""),
    ))
}
