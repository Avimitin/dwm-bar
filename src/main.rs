mod component;

/// Reset the color the SchemeNorm
static NORMAL_COLOR: &str = "^d^";
static DIVIDER: &str = "     |     ";

use anyhow::Result;
use argh::FromArgs;
use std::{process::Command, sync::Arc, time::Duration};
use tokio::spawn as t_spawn;

#[cfg(feature = "bluetooth-battery")]
use tokio::time::interval;
#[cfg(feature = "bluetooth-battery")]
use tokio::sync::watch;

use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(FromArgs)]
/// Print computer status to dwm bar
struct App {
    #[argh(switch)]
    /// run this command only one time for testing
    once: bool,

    #[argh(switch)]
    /// do not output contents to the bar, print it to stdout instead
    dry: bool,
}

#[cfg(feature = "bluetooth-battery")]
use component::Block;

#[cfg(feature = "bluetooth-battery")]
async fn function(tx: watch::Sender<Option<Block>>, secs: u64) -> Result<()> {
    let mut headset_battery = component::HeadsetBattery::new().await?;

    let mut ticker = interval(Duration::from_secs(secs));
    loop {
        if tx.is_closed() {
            break;
        }

        let block = headset_battery.update().await;
        tx.send(block)?;

        ticker.tick().await;
    }

    Ok(())
}

async fn run(app: &App) -> Result<()> {
    info!("Entering information fetching loop");

    let song_info = Arc::new(component::SongInfo::new()?);

    #[cfg(feature = "bluetooth-battery")]
    let (tx, rx) = watch::channel(None);
    #[cfg(feature = "bluetooth-battery")]
    t_spawn(async move {
        function(tx, 10).await.unwrap();
    });

    loop {
        let song = song_info.clone();
        #[cfg(feature = "bluetooth-battery")]
        let mut btbat_rx = rx.clone();
        let bar = vec![
            t_spawn(async move { song.song_info().await }),
            t_spawn(async { component::sound_volume().await }),
            #[cfg(feature = "bluetooth-battery")]
            t_spawn(async move {
                if btbat_rx.changed().await.is_ok() {
                    (*btbat_rx.borrow()).clone()
                } else {
                    None
                }
            }),
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

        if !app.dry {
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
        } else {
            info!("New output: {}", barline);
        }

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
