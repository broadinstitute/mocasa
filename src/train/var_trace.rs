use crate::math::stats::Stats;
use crate::mcmc::metro::Draw;

pub(crate) struct VarTracer {
    stats: Stats
}

const REFRESH_THRESHOLD: usize = 14;

impl VarTracer {
    pub(crate) fn new(mean_estimate: f64, std_dev_estimate: f64) -> VarTracer {
        let mut stats = Stats::new();
        stats.add(mean_estimate - std_dev_estimate);
        stats.add(mean_estimate + std_dev_estimate);
        VarTracer { stats }
    }
    pub(crate) fn add(&mut self, draw: Draw) {
        self.stats.add(draw.x);
        if self.stats.n > REFRESH_THRESHOLD && draw.attempts_minus > REFRESH_THRESHOLD &&
            draw.attempts_plus > REFRESH_THRESHOLD {
            self.refresh();
        };
    }
    pub(crate) fn refresh(&mut self) {
        let mean = self.stats.mean().unwrap();
        let std_dev = self.stats.variance().unwrap().sqrt();
        self.stats.reset();
        self.stats.add(mean - std_dev);
        self.stats.add(mean + std_dev)
    }
    pub(crate) fn std_dev(&self) -> f64 {
        self.stats.variance().unwrap().sqrt()
    }
}