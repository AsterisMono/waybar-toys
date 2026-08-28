//! Command line configuration (using clap). All settings are flags so the
//! whole configuration lives in the waybar/polybar `exec` line — no config file.

use clap::Parser;

const DEFAULT_FRAMES: &str = "\u{E905}\u{E904}\u{E903}\u{E902}\u{E901}";
const DEFAULT_SLEEPING_FRAMES: &str = "\u{E900}\u{E8FF}";

#[derive(Parser, Debug)]
#[command(
    name = "waycat",
    version,
    about = "runcat module for polybar (or waybar)"
)]
pub struct Args {
    /// Stat file used to poll the CPU.
    #[arg(short, long, default_value = "/proc/stat")]
    pub stat_path: String,

    /// Frames to loop through (non-empty).
    #[arg(long, default_value = DEFAULT_FRAMES)]
    pub frames: String,

    /// Maximum FPS.
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub high_rate: u8,

    /// Minimum FPS.
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub low_rate: u8,

    /// CPU polling period in milliseconds.
    #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(1..))]
    pub poll_period: u64,

    /// Enable smooth transitions between CPU load values.
    #[arg(long, default_value_t = true, num_args = 0..=1, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
    pub smoothing_enabled: bool,

    /// Milliseconds to transition from 0% to 100%.
    #[arg(long, default_value_t = 2000, value_parser = clap::value_parser!(u64).range(1..=10000))]
    pub smoothing_value: u64,

    /// Enable sleeping (idle animation).
    #[arg(long, default_value_t = true, num_args = 0..=1, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
    pub sleeping_enabled: bool,

    /// Max load to enter sleeping mode (percent).
    #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub sleeping_threshold: u8,

    /// Max load to remain in sleeping mode (percent).
    #[arg(long, default_value_t = 12, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub wakeup_threshold: u8,

    /// Frames to loop in sleeping mode (non-empty).
    #[arg(long, default_value = DEFAULT_SLEEPING_FRAMES)]
    pub sleeping_frames: String,

    /// FPS for the sleeping animation.
    #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub sleeping_rate: u8,

    /// Enable output formatting.
    #[arg(long, default_value_t = false, num_args = 0..=1, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
    pub format_enabled: bool,

    /// Format string: $frame, $lcpu, $rcpu, $$ (escape a literal $).
    #[arg(long, default_value = "$frame $lcpu")]
    pub format: String,
}

impl Args {
    /// Cross-field validation that clap cannot express.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.frames.is_empty() {
            errors.push("`frames` cannot be an empty string".into());
        }
        if self.sleeping_frames.is_empty() {
            errors.push("`sleeping_frames` cannot be an empty string".into());
        }
        if self.sleeping_threshold > self.wakeup_threshold {
            errors.push("`wakeup_threshold` should be greater than `sleeping_threshold`".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}