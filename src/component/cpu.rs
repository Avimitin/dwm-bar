use super::widget::Block;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

pub async fn avg_load() -> Option<Block> {
    let status = fs::read_to_string("/proc/stat").await.ok()?;
    let mut before = Vec::new();
    for line in status.lines() {
        if line.starts_with("cpu") {
            // remove the "cpu" prefix
            before = line
                .split_whitespace()
                .skip(1)
                .map(|x| {
                    x.parse::<i32>()
                        .expect("Fail to parse the /proc/stat file, please check your system")
                })
                .collect();
            break;
        }
    }

    if before.len() < 8 {
        return None;
    }

    sleep(Duration::from_secs(1)).await;

    let status = fs::read_to_string("/proc/stat").await.ok()?;
    let mut after = Vec::new();
    for line in status.lines() {
        if line.starts_with("cpu") {
            // remove the "cpu" prefix
            after = line
                .split_whitespace()
                .skip(1)
                .map(|x| {
                    x.parse::<i32>()
                        .expect("Fail to parse the /proc/stat file, please check your system")
                })
                .collect();
            break;
        }
    }

    if after.len() < 8 {
        return None;
    }

    // get the cpu idle time
    let before_sum: i32 = before.iter().sum();
    let after_sum: i32 = after.iter().sum();

    let sum = (before_sum - after_sum) as f32;

    let before_idle: i32 = before[3];
    let before_iowait: i32 = before[4];
    let before_sum = (before_sum - before_idle - before_iowait) as f32;

    let after_idle: i32 = after[3];
    let after_iowait: i32 = after[4];
    let after_sum = (after_sum - after_idle - after_iowait) as f32;

    let avg = (before_sum - after_sum) / sum;

    Some(
        Block::new("﬙", format!("{:.2} %", avg * 100.0))
            .text_fg("#EAEAEA")
            .icon_fg("#EAEAEA"),
    )
}

#[tokio::test]
async fn test() {
    dbg!(avg_load().await);
}
