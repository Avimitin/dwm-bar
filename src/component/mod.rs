mod color;
mod widget;

mod song;
mod datetime;
mod volume;
mod battery;
mod cpu;
#[cfg(feature = "bluetooth-battery")]
mod headset_battery;

// re-export
pub use song::SongInfo;
pub use datetime::date_and_time;
pub use volume::sound_volume;
#[cfg(feature = "bluetooth-battery")]
pub use headset_battery::HeadsetBattery;
pub use battery::battery;
pub use cpu::avg_load;
pub use widget::Block;
