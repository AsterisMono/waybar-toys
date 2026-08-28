//! CPU usage polling (port of `src/cpu.*`).

use std::fmt;

#[derive(Debug)]
pub enum CpuError {
    Io(String),
    Fmt(String),
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuError::Io(m) | CpuError::Fmt(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct State {
    total: u64,
    work: u64,
}

pub struct Cpu {
    stat_path: String,
    state_prev: State,
}

impl Cpu {
    pub fn new(stat_path: &str) -> Self {
        Cpu {
            stat_path: stat_path.to_string(),
            state_prev: State::default(),
        }
    }

    pub fn poll(&mut self) -> Result<f32, CpuError> {
        let state_curr = self.get_state()?;

        let work_d = state_curr.work - self.state_prev.work;
        let total_d = state_curr.total - self.state_prev.total;

        self.state_prev = state_curr;

        Ok(work_d as f32 / total_d as f32)
    }

    fn get_state(&self) -> Result<State, CpuError> {
        let content = std::fs::read_to_string(&self.stat_path)
            .map_err(|_| CpuError::Io("Failed to open the stat file.".to_string()))?;

        let line = match content.lines().next() {
            Some(l) => l,
            None => return Err(CpuError::Fmt("Stat file is empty.".to_string())),
        };

        let mut tokens = line.split_whitespace();

        match tokens.next() {
            Some("cpu") => {}
            Some(_) => return Err(CpuError::Fmt("Stat has invalid format.".to_string())),
            None => return Err(CpuError::Fmt("Stat file is empty.".to_string())),
        }

        let mut jiffies = Vec::with_capacity(16);
        for token in tokens {
            match token.parse::<u64>() {
                Ok(v) => jiffies.push(v),
                Err(_) => return Err(CpuError::Fmt("Stat file has invalid data.".to_string())),
            }
        }

        if jiffies.len() < 4 {
            return Err(CpuError::Fmt("Not enough data in stat file.".to_string()));
        }

        let total: u64 = jiffies.iter().sum();
        let work: u64 = jiffies.iter().take(3).sum();

        Ok(State { total, work })
    }
}
