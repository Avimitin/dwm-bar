/// Create a date component for bar
use super::widget::Block;
use chrono::prelude::Local;

pub struct DateTime {
    format: String,
    icon: String,
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            format: "%B/%d %I:%M %p".to_string(),
            icon: "".to_string(),
        }
    }
}

impl DateTime {
    fn new<T: Into<String>, K: Into<String>>(icon: T, format: K) -> Self {
        Self { icon: icon.into(), format: format.into() }
    }
}

#[async_trait::async_trait]
impl super::Updater for DateTime {
    async fn update(&mut self) -> Option<Block> {
        let now = Local::now();
        Some(
            Block::new(&self.icon, now.format(&self.format).to_string())
                .text_fg("#EAEAEA")
                .icon_fg("#EAEAEA"),
        )
    }
}
