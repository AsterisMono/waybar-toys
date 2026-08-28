//! Frame cycler (port of `src/framer.*`).

pub struct Framer {
    frames: Vec<char>,
    curr: usize,
}

impl Framer {
    pub fn new(frames: &str) -> Self {
        Framer {
            frames: frames.chars().collect(),
            curr: 0,
        }
    }

    /// Returns the current frame and advances to the next one.
    pub fn get(&mut self) -> String {
        let count = self.frames.len();
        if count == 0 {
            // Should be impossible after config validation; guard anyway.
            self.curr = 0;
            return String::new();
        }
        let frame = self.frames[self.curr].to_string();
        self.curr = (self.curr + 1) % count;
        frame
    }
}
