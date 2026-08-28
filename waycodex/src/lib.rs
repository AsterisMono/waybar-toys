use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UsageResponse {
    pub rate_limit: RateLimit,
    #[serde(default)]
    pub rate_limit_reset_credits: Option<ResetCreditSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimit {
    pub primary_window: Option<Window>,
    pub secondary_window: Option<Window>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    pub used_percent: f64,
    pub limit_window_seconds: Option<u64>,
    pub reset_at: Option<i64>,
    pub reset_after_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetCreditSummary {
    pub available_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetCreditsResponse {
    pub available_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayUsage {
    pub five_hour_used: f64,
    pub week_used: f64,
    pub banked: Option<u64>,
    pub five_hour_reset: Option<i64>,
    pub week_reset: Option<i64>,
}

impl DisplayUsage {
    pub fn from_response(response: &UsageResponse, banked: Option<u64>, now: i64) -> Option<Self> {
        let windows = [
            response.rate_limit.primary_window.as_ref(),
            response.rate_limit.secondary_window.as_ref(),
        ];
        let mut available: Vec<&Window> = windows.into_iter().flatten().collect();
        if available.len() < 2 {
            return None;
        }
        available.sort_by_key(|window| window.limit_window_seconds.unwrap_or(u64::MAX));
        let five_hour = available[0];
        let week = available[1];
        Some(Self {
            five_hour_used: five_hour.used_percent.clamp(0.0, 100.0),
            week_used: week.used_percent.clamp(0.0, 100.0),
            banked: banked.or_else(|| {
                response
                    .rate_limit_reset_credits
                    .as_ref()
                    .map(|credits| credits.available_count)
            }),
            five_hour_reset: reset_timestamp(five_hour, now),
            week_reset: reset_timestamp(week, now),
        })
    }

    pub fn next_reset(&self) -> Option<i64> {
        [self.five_hour_reset, self.week_reset]
            .into_iter()
            .flatten()
            .min()
    }

    pub fn render(&self, template: &str, now: i64) -> String {
        let banked = self
            .banked
            .map_or_else(|| "?".into(), |value| value.to_string());
        let values = [
            ("{5h_used}", format_percent(self.five_hour_used)),
            ("{1w_used}", format_percent(self.week_used)),
            ("{5h_left}", format_percent(100.0 - self.five_hour_used)),
            ("{1w_left}", format_percent(100.0 - self.week_used)),
            ("{banked}", banked),
            ("{next_reset}", format_reset(self.next_reset(), now)),
            ("{5h_reset}", format_reset(self.five_hour_reset, now)),
            ("{1w_reset}", format_reset(self.week_reset, now)),
        ];
        values
            .into_iter()
            .fold(template.to_string(), |text, (token, value)| {
                text.replace(token, &value)
            })
    }
}

fn reset_timestamp(window: &Window, now: i64) -> Option<i64> {
    window.reset_at.or_else(|| {
        window
            .reset_after_seconds
            .map(|seconds| now.saturating_add(seconds as i64))
    })
}

fn format_percent(value: f64) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

pub fn format_reset(timestamp: Option<i64>, now: i64) -> String {
    let Some(timestamp) = timestamp else {
        return "?".into();
    };
    let seconds = timestamp.saturating_sub(now);
    if seconds <= 0 {
        return "now".into();
    }
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "<1m".into()
    }
}

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(used: f64, seconds: u64, reset: i64) -> Window {
        Window {
            used_percent: used,
            limit_window_seconds: Some(seconds),
            reset_at: Some(reset),
            reset_after_seconds: None,
        }
    }

    #[test]
    fn identifies_windows_by_duration_and_renders() {
        let response = UsageResponse {
            rate_limit: RateLimit {
                primary_window: Some(window(12.5, 604_800, 200_000)),
                secondary_window: Some(window(34.0, 18_000, 103_660)),
            },
            rate_limit_reset_credits: Some(ResetCreditSummary { available_count: 2 }),
        };
        let usage = DisplayUsage::from_response(&response, None, 100_000).unwrap();
        assert_eq!(usage.five_hour_used, 34.0);
        assert_eq!(usage.week_used, 12.5);
        assert_eq!(usage.banked, Some(2));
        assert_eq!(
            usage.render(
                "5h {5h_used}% 1w {1w_used}% R{banked} {next_reset}",
                100_000
            ),
            "5h 34% 1w 12.5% R2 1h 1m"
        );
    }

    #[test]
    fn formats_reset_countdowns() {
        assert_eq!(format_reset(Some(100), 100), "now");
        assert_eq!(format_reset(Some(159), 100), "<1m");
        assert_eq!(format_reset(Some(3_760), 100), "1h 1m");
        assert_eq!(format_reset(Some(93_700), 100), "1d 2h");
        assert_eq!(format_reset(None, 100), "?");
    }
}
