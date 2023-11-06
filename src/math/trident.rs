use std::mem::replace;
use crate::math::skip_stats::SkipStats;

#[derive(Clone)]
pub(crate) struct TridentStats {
    stats: [SkipStats; 3],
}

impl TridentStats {
    pub(crate) fn new(x0: f64, x1: f64) -> TridentStats {
        let skip_stats = SkipStats::new(x0, x1);
        let stats = [skip_stats.clone(), skip_stats.clone(), skip_stats];
        TridentStats { stats }
    }
    pub(crate) fn add(&mut self, x: f64) {
        self.stats[2].add(x);
        if self.stats[2].n >= 2 * self.stats[1].n {
            let mean = self.mean();
            let std_dev = self.std_dev();
            let x0 = mean - std_dev;
            let x1 = mean + std_dev;
            let stats_new = SkipStats::new(x0, x1);
            let stats_old_2 = replace(&mut self.stats[2], stats_new);
            let stats_old_1 = replace(&mut self.stats[1], stats_old_2);
            let _ = replace(&mut self.stats[0], stats_old_1);
        }
    }
    pub(crate) fn mean(&self) -> f64 {
        (self.stats[0].sum + self.stats[1].sum + self.stats[2].sum)
            / ((self.stats[0].n + self.stats[1].n + self.stats[2].n) as f64)
    }
    pub(crate) fn variance(&self) -> f64 {
        (self.stats[0].var_sum + self.stats[1].var_sum + self.stats[2].var_sum)
            / ((self.stats[0].n + self.stats[1].n + self.stats[2].n) as f64)
    }
    pub(crate) fn std_dev(&self) -> f64 { self.variance().sqrt() }
}

#[cfg(test)]
mod tests {
    use crate::math::trident::TridentStats;

    #[test]
    fn collect_stats() {
        let mut stats = TridentStats::new(100.0, 100.1);
        for i in 0..2500 {
            let x = i as f64;
            let f = 100.0 * f64::exp(-0.01 * x) + f64::sin(0.1 * x);
            stats.add(f);
            println!("i = {},\tf = {},\tmean = {},\tstd_dev = {},\t({},{},{})", i, f, stats.mean(),
                     stats.std_dev(), stats.stats[0].n, stats.stats[1].n, stats.stats[2].n);
        }
    }
}