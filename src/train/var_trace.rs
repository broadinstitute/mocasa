use crate::math::stats::Stats;
use crate::mcmc::metro::Draw;

pub(crate) struct VarTrace {
    stats: Stats,
    n_swap: usize
}

impl VarTrace {
    pub(crate) fn new(mean_initial: f64, std_dev_initial: f64, n_swap: usize) -> VarTrace {
        let mut stats = Stats::new();
        stats.add(mean_initial - std_dev_initial);
        stats.add(mean_initial + std_dev_initial);
        VarTrace { stats, n_swap }
    }
    pub(crate) fn add(&mut self, draw: Draw) {
        self.stats.add(draw.x);
        if self.stats.n > self.n_swap {
            let mean = self.stats.mean().unwrap();
            let std_dev = self.stats.variance().unwrap().sqrt();
            self.stats.reset();
            self.stats.add(mean - std_dev);
            self.stats.add(mean + std_dev)
        };
    }
}