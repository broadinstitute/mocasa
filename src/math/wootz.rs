use std::mem;
use crate::math::skip_stats::SkipStats;

#[derive(Clone)]
pub(crate) struct WootzStats {
    stats: SkipStats,
    snapshots: Vec<SkipStats>,
    n_per_snapshot: usize,
    burnedness: usize
}

const N_PER_SNAPSHOT_INITIAL: usize = 10;
const N_SNAPSHOTS_MAX: usize = 10;

impl WootzStats {
    pub(crate) fn new(x0: f64, x1: f64) -> WootzStats {
        let stats = SkipStats::new(x0, x1);
        let snapshots: Vec<SkipStats> = Vec::new();
        let n_per_snapshot = N_PER_SNAPSHOT_INITIAL;
        let burnedness: usize = 0;
        WootzStats { stats, snapshots, n_per_snapshot, burnedness }
    }
    pub(crate) fn add(&mut self, x_new: f64) {
        self.stats.add(x_new);
        if self.stats.n % self.n_per_snapshot == 0 {
            self.snapshots.push(self.stats.clone());
            if self.snapshots.len() > N_SNAPSHOTS_MAX {
                if self.should_truncate() {
                    self.truncate();
                    self.burnedness = 0;
                } else {
                    self.fold();
                    self.burnedness += 1;
                }
            }
        }
    }
    pub(crate) fn n(&self) -> usize { self.stats.n }
    pub(crate) fn mean(&self) -> f64 { self.stats.mean() }
    pub(crate) fn variance(&self) -> f64 { self.stats.variance() }
    pub(crate) fn std_dev(&self) -> f64 { self.variance().sqrt() }
    pub(crate) fn squash(&mut self) {
        let mean = self.mean();
        let std_dev = self.std_dev();
        let x0 = mean - std_dev;
        let x1 = mean + std_dev;
        self.stats = SkipStats::new(x0, x1);
        self.snapshots = Vec::new();
        self.n_per_snapshot = N_PER_SNAPSHOT_INITIAL;
    }
    pub(crate) fn autocity(&self) -> Option<f64> {
        let n_snapshots = self.snapshots.len();
        if n_snapshots < 3 {
            None
        } else {
            let mean = self.stats.mean();
            let mut buckets_var_sum: f64 = (self.snapshots[0].mean() - mean).powi(2);
            for pair in self.snapshots.windows(2) {
                let [s0, s1] = pair else { unreachable!() };
                let bucket_stats = s1.try_minus(s0).unwrap();
                buckets_var_sum += (bucket_stats.mean() - mean).powi(2);
            }
            let bucket_var = buckets_var_sum / (n_snapshots as f64);
            let autocity = (self.n_per_snapshot as f64) * bucket_var / self.variance();
            Some(autocity)
        }
    }
    pub(crate) fn burnedness(&self) -> usize { self.burnedness }
    fn should_truncate(&self) -> bool {
        let mean = self.stats.mean();
        let dist0 = (self.snapshots[0].mean() - mean).abs();
        self.snapshots.windows(2).all(|pair| {
            let [s0, s1] = pair else { unreachable!() };
            let n = s1.n - s0.n;
            let sum = s1.sum - s0.sum;
            let mean_layer = sum / (n as f64);
            let dist_layer = (mean_layer - mean).abs();
            dist_layer < dist0
        })
    }
    fn truncate(&mut self) {
        let mut iter = self.snapshots.iter();
        let s0 = iter.next().unwrap();
        self.stats = self.stats.try_minus(s0).unwrap();
        self.snapshots = iter.map(|s| s.try_minus(s0).unwrap()).collect();
    }
    fn fold(&mut self) {
        let snapshots = mem::take(&mut self.snapshots);
        self.snapshots =
            snapshots.into_iter().enumerate().filter(|(i, _)| *i % 2 != 0)
                .map(|(_, stats)| stats).collect();
        self.n_per_snapshot *= 2;
    }
}
