mod component;

/// Reset the color the SchemeNorm
static NORMAL_COLOR: &str = "^d^";
static DIVIDER: &str = "     |     ";

use std::process::{exit, Command};
use std::time::Duration;
use tokio::spawn as t_spawn;
use tokio::time::sleep;

async fn run() {
    let bar = vec![
        t_spawn(async { component::song_info().await }),
        t_spawn(async { component::sound_volume().await }),
        #[cfg(feature = "bluetooth-battery")]
        t_spawn(async { component::headset_battery().await }),
        t_spawn(async { component::battery().await }),
        t_spawn(async { component::avg_load().await }),
        t_spawn(async { component::date_and_time() }),
    ];

    let mut info = Vec::new();
    for task in bar {
        let i = task.await.unwrap();
        info.push(i);
    }

    let mut begining = true;
    let mut barline = String::new();
    for component in info.iter().flatten() {
        if begining {
            begining = false;
        } else {
            barline.push_str(DIVIDER);
        }
        barline.push_str(&format!("{}", component));
        barline.push_str(NORMAL_COLOR);
    }

    // Clean the bar
    let mut cmd = Command::new("xsetroot");
    let _hold = cmd
        .arg("-name")
        .arg("''")
        .output()
        .expect("Fail to execute xsetroot");
    let _hold = cmd
        .arg("-name")
        .arg(barline)
        .output()
        .expect("Fail to execute xsetroot");
}

use argh::FromArgs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(FromArgs)]
/// Print computer status to dwm bar
struct App {
    #[argh(switch, short = 'd')]
    /// run this command only one time for testing
    dry: bool,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Fail to set default logger");

    let app: App = argh::from_env();
    // run once
    if app.dry {
        run().await;
        exit(0);
    }

    info!("Entering information fetching loop");
    loop {
        run().await;
        sleep(Duration::from_secs(10)).await;
    }
}
