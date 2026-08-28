//! Output formatter (port of `src/formatter.*`).

use std::fmt;

const FORMAT_PREFIX: &str = "$";
const R_CPU_LOAD_KEY: &str = "rcpu";
const L_CPU_LOAD_KEY: &str = "lcpu";
const FRAME_KEY: &str = "frame";

pub struct Formatter {
    format: String,
    rcpu_fmt: String,
    lcpu_fmt: String,
    frame_fmt: String,
    prefix_fmt: String,
}

#[derive(Debug)]
pub struct FmtErr(pub String);

impl fmt::Display for FmtErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FmtErr {}

fn count_occurrences(string: &str, substring: &str) -> usize {
    if substring.is_empty() {
        return 0;
    }
    string.matches(substring).count()
}

fn replace_all(string: String, from: &str, to: &str) -> String {
    if from.is_empty() {
        return string;
    }
    string.replace(from, to)
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            format: String::new(),
            rcpu_fmt: String::new(),
            lcpu_fmt: String::new(),
            frame_fmt: String::new(),
            prefix_fmt: String::new(),
        }
    }

    pub fn set(&mut self, format: &str) -> Result<(), FmtErr> {
        let rcpu_fmt = format!("{}{}", FORMAT_PREFIX, R_CPU_LOAD_KEY);
        let lcpu_fmt = format!("{}{}", FORMAT_PREFIX, L_CPU_LOAD_KEY);
        let frame_fmt = format!("{}{}", FORMAT_PREFIX, FRAME_KEY);
        let prefix_fmt = format!("{}{}", FORMAT_PREFIX, FORMAT_PREFIX);

        let prefix_occurrences = count_occurrences(format, FORMAT_PREFIX);
        let rcpu_occurrences = count_occurrences(format, &rcpu_fmt);
        let lcpu_occurrences = count_occurrences(format, &lcpu_fmt);
        let frame_occurrences = count_occurrences(format, &frame_fmt);
        let esc_prefix_occurrences = count_occurrences(format, &prefix_fmt);

        let prefix_occurrences = prefix_occurrences - esc_prefix_occurrences;

        if prefix_occurrences
            != rcpu_occurrences + lcpu_occurrences + frame_occurrences + esc_prefix_occurrences
        {
            return Err(FmtErr(format!(
                "String \"{}\" has incorrect format.",
                format
            )));
        }

        self.format = format.to_string();
        self.rcpu_fmt = rcpu_fmt;
        self.lcpu_fmt = lcpu_fmt;
        self.frame_fmt = frame_fmt;
        self.prefix_fmt = prefix_fmt;

        Ok(())
    }

    pub fn format(&self, frame: &str, load: u8) -> String {
        let mut result = self.format.clone();

        let mut r_load = format!("{}%", load as u32);
        let mut l_load = r_load.clone();

        while l_load.chars().count() < 4 && r_load.chars().count() < 4 {
            r_load = format!(" {}", r_load);
            l_load = format!("{} ", l_load);
        }

        result = replace_all(result, &self.prefix_fmt, FORMAT_PREFIX);
        result = replace_all(result, &self.frame_fmt, frame);
        result = replace_all(result, &self.rcpu_fmt, &r_load);
        result = replace_all(result, &self.lcpu_fmt, &l_load);

        result
    }
}
