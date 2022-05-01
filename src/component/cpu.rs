use super::widget::Block;
use tokio::fs;

pub struct Cpu {
    last: Option<CpuStat>,
    icon: String,
}

impl Cpu {
    pub async fn new(icon: &str) -> Self {
        Self {
            last: CpuStat::new().await,
            icon: icon.into(),
        }
    }
}

#[async_trait::async_trait]
impl super::Updater for Cpu {
    async fn update(&mut self) -> Option<Block> {
        let before = self.last?;

        let after = CpuStat::new().await?;

        // get total
        let sum = (before.sum - after.sum) as f32;
        // use active time / total
        let avg = ((before.active - after.active) as f32) / sum;

        self.last.replace(after);

        Some(
            Block::new("﬙", format!("{:.2} %", avg * 100.0))
                .text_fg("#EAEAEA")
                .icon_fg("#EAEAEA"),
        )
    }
}

struct CpuStat {
    sum: i32,
    active: i32,
}

impl CpuStat {
    async fn new() -> Option<Self> {
        let status = fs::read_to_string("/proc/stat").await.ok()?;
        let mut time = Vec::new();
        for line in status.lines() {
            if line.starts_with("cpu") {
                // remove the "cpu" prefix
                time = line
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

        if time.len() < 8 {
            return None;
        }

        let sum: i32 = time.iter().sum();
        // idle + iowait
        let inactive = time[3] + time[4];

        Some(Self {
            sum,
            active: sum - inactive,
        })
    }
}
