//! CPU-load smoother (port of `src/smoother.*`).

pub struct Smoother {
    period: u64,
    target: f32,
    prev: f32,
}

impl Smoother {
    pub fn new(period: u64) -> Self {
        Smoother {
            period,
            target: 0.0,
            prev: 0.0,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Returns a smoothed value advanced by `delta` milliseconds.
    pub fn value(&mut self, delta: u64) -> f32 {
        if self.prev == self.target {
            return self.target;
        }

        let diff = self.target - self.prev;
        let sign = if diff > 0.0 {
            1.0f32
        } else if diff < 0.0 {
            -1.0f32
        } else {
            0.0f32
        };
        let step = delta as f32 / self.period as f32;

        let mut result = self.prev + step * sign;
        result = if sign > 0.0 {
            result.min(self.target)
        } else {
            result.max(self.target)
        };

        self.prev = result;
        result
    }
}
