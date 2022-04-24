mod component;

/// Reset the color the SchemeNorm
static NORMAL_COLOR: &str = "^d^";
static DIVIDER: &str = "     |     ";

use anyhow::Result;
use argh::FromArgs;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn as t_spawn;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(FromArgs)]
/// Print computer status to dwm bar
struct App {
    #[argh(switch)]
    /// run this command only one time for testing
    once: bool,
}

async fn run(app: &App) -> Result<()> {
    info!("Entering information fetching loop");

    let song_info = Arc::new(component::SongInfo::new()?);

    loop {
        let song = song_info.clone();
        let bar = vec![
            t_spawn(async move { song.song_info().await }),
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

        if app.once {
            return Ok(());
        }

        sleep(Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Fail to set default logger");

    let app: App = argh::from_env();
    run(&app).await.unwrap();
}
