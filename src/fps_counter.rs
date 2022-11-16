use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct FpsCounter {
    frames: VecDeque<Instant>,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::default(),
        }
    }

    pub fn tick(&mut self) -> usize {
        let now = Instant::now();
        self.frames.push_back(now);
        let one_second_from_now = now - Duration::from_secs(1);

        while self
            .frames
            .front()
            .map_or(false, |t| t < &one_second_from_now)
        {
            self.frames.pop_front();
        }

        self.frames.len()
    }
}
