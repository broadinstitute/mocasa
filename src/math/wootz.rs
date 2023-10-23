use crate::math::skip_stats::SkipStats;

pub(crate) struct WootzStats {
    stats: SkipStats,
    snapshots: Vec<SkipStats>,
    n_per_snapshot: usize
}

const N_PER_SNAPSHOT_INITIAL: usize = 10;

impl WootzStats {
    pub(crate) fn new(x0: f64, x1: f64) -> WootzStats {
        let main_layer = SkipStats::new(x0, x1);
        let snapshots: Vec<SkipStats> = Vec::new();
        let n_per_snapshot = N_PER_SNAPSHOT_INITIAL;
        WootzStats { stats: main_layer, snapshots, n_per_snapshot }
    }
    pub(crate) fn add(&mut self, x_new: f64) {
        self.stats.add(x_new);
        if self.stats.n % self.n_per_snapshot == 0 {
            self.snapshots.push(self.stats.clone());

        }
    }
}