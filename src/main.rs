mod component;

/// Reset the color the SchemeNorm
static NORMAL_COLOR: &str = "^d^";
static DIVIDER: &str = "     |     ";

use std::process::{exit, Command};
use tokio::time::sleep;
use std::time::Duration;

async fn run() {
    let bar = vec![
        component::song_info().await,
        component::sound_volume().await,
        #[cfg(feature = "bluetooth-battery")]
        component::headset_battery(),
        component::battery().await,
        component::avg_load().await,
        component::date_and_time(),
    ];


    let mut begining = true;
    let mut barline = String::new();
    for component in bar.iter().flatten() {
        if begining {
            begining = false;
        } else {
            barline.push_str(DIVIDER);
        }
        barline.push_str(&format!("{}", component));
        barline.push_str(NORMAL_COLOR);
    }

    if let Ok(mut child) = Command::new("xsetroot").arg("-name").arg(barline).spawn() {
        child.wait().expect("fail to end the xsetroot command");
    } else {
        eprintln!("Fail to execute xsetroot")
    }
}

use argh::FromArgs;

#[derive(FromArgs)]
/// Print computer status to dwm bar
struct App {
    #[argh(switch, short = 'd')]
    /// run this command only one time for testing
    dry: bool,
}

#[tokio::main]
async fn main() {
    let app: App = argh::from_env();
    // run once
    if app.dry {
        run().await;
        exit(0);
    }

    loop {
        run().await;
        sleep(Duration::from_secs(10)).await;
    }
}
