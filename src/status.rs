//! Agent lifecycle statuses as reported by the herdr server, plus
//! aggregation and bar-text rendering.

/// Herdr agent lifecycle state.
///
/// `unknown` is herdr's catch-all: an agent is present but could not be
/// classified confidently. It is mapped from any unrecognized wire value so
/// future protocol additions do not break rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Working,
    Blocked,
    Done,
    Idle,
    Unknown,
}

impl AgentStatus {
    pub const ALL: [AgentStatus; 5] = [
        AgentStatus::Working,
        AgentStatus::Blocked,
        AgentStatus::Done,
        AgentStatus::Idle,
        AgentStatus::Unknown,
    ];

    /// Parse a wire value (`"idle"`, `"working"`, ...), defaulting to
    /// [`AgentStatus::Unknown`].
    pub fn from_raw(value: &str) -> AgentStatus {
        match value {
            "working" => AgentStatus::Working,
            "blocked" => AgentStatus::Blocked,
            "done" => AgentStatus::Done,
            "idle" => AgentStatus::Idle,
            _ => AgentStatus::Unknown,
        }
    }

    /// Wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            AgentStatus::Working => "working",
            AgentStatus::Blocked => "blocked",
            AgentStatus::Done => "done",
            AgentStatus::Idle => "idle",
            AgentStatus::Unknown => "unknown",
        }
    }
}

/// Per-status agent counts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub working: usize,
    pub blocked: usize,
    pub done: usize,
    pub idle: usize,
    pub unknown: usize,
}

impl Counts {
    pub fn add(&mut self, status: AgentStatus) {
        match status {
            AgentStatus::Working => self.working += 1,
            AgentStatus::Blocked => self.blocked += 1,
            AgentStatus::Done => self.done += 1,
            AgentStatus::Idle => self.idle += 1,
            AgentStatus::Unknown => self.unknown += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.working + self.blocked + self.done + self.idle + self.unknown
    }

    /// Substitute `{working}`, `{blocked}`, `{done}`, `{idle}`, `{unknown}`
    /// and `{total}` tokens in `template`. Unknown `{...}` tokens are left
    /// untouched.
    pub fn render(&self, template: &str) -> String {
        template
            .replace("{working}", &self.working.to_string())
            .replace("{blocked}", &self.blocked.to_string())
            .replace("{done}", &self.done.to_string())
            .replace("{idle}", &self.idle.to_string())
            .replace("{unknown}", &self.unknown.to_string())
            .replace("{total}", &self.total().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_statuses() {
        assert_eq!(AgentStatus::from_raw("working"), AgentStatus::Working);
        assert_eq!(AgentStatus::from_raw("blocked"), AgentStatus::Blocked);
        assert_eq!(AgentStatus::from_raw("done"), AgentStatus::Done);
        assert_eq!(AgentStatus::from_raw("idle"), AgentStatus::Idle);
    }

    #[test]
    fn unknown_values_fall_back_to_unknown() {
        assert_eq!(AgentStatus::from_raw("napping"), AgentStatus::Unknown);
        assert_eq!(AgentStatus::from_raw(""), AgentStatus::Unknown);
    }

    #[test]
    fn as_str_round_trips() {
        for status in AgentStatus::ALL {
            assert_eq!(AgentStatus::from_raw(status.as_str()), status);
        }
    }

    #[test]
    fn counts_accumulate_and_total() {
        let mut counts = Counts::default();
        for status in [
            AgentStatus::Working,
            AgentStatus::Working,
            AgentStatus::Blocked,
            AgentStatus::Idle,
            AgentStatus::Done,
            AgentStatus::Unknown,
        ] {
            counts.add(status);
        }
        assert_eq!(counts.working, 2);
        assert_eq!(counts.blocked, 1);
        assert_eq!(counts.done, 1);
        assert_eq!(counts.idle, 1);
        assert_eq!(counts.unknown, 1);
        assert_eq!(counts.total(), 6);
    }

    #[test]
    fn render_substitutes_tokens() {
        let mut counts = Counts::default();
        counts.add(AgentStatus::Working);
        counts.add(AgentStatus::Idle);
        counts.add(AgentStatus::Blocked);
        counts.add(AgentStatus::Done);

        let text = counts.render("{working}w {blocked}b {done}d {idle}i");
        assert_eq!(text, "1w 1b 1d 1i");
    }

    #[test]
    fn render_includes_total_and_preserves_unknown_tokens() {
        let counts = Counts {
            working: 2,
            idle: 3,
            ..Default::default()
        };
        assert_eq!(counts.render("{total} total"), "5 total");
        assert_eq!(counts.render("{nope} {working}"), "{nope} 2");
    }
}
