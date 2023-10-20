use crate::math::stats::Stats;

struct Layer {
    n: usize,
    sum: f64,
    var_sum: f64
}

pub(crate) struct WootzStats {
    main_layer: Layer,
    snapshots: Vec<Stats>,
    n_per_snapshot: usize
}

const N_PER_SNAPSHOT_INITIAL: usize = 10;

impl Layer {
    fn new(mean_initial: f64, std_dev_initial: f64) -> Layer {
        let n: usize = 0;
        let sum: f64 = 0.0;
        let var_sum: f64 = 0.0;
        Layer { n, sum, var_sum }
    }
}

impl WootzStats {
    pub(crate) fn new(mean_initial: f64, std_dev_initial: f64) -> WootzStats {
        let stats = Stats::new();
        let snapshots: Vec<Stats> = Vec::new();
        let n_per_snapshot = N_PER_SNAPSHOT_INITIAL;
        WootzStats { main_layer: stats, snapshots, n_per_snapshot }
    }
    pub(crate) fn add(&mut self, x_new: f64) {
        if self.main_layer.n < self.n_per_layer {
            self.main_layer.add(x_new)
        } else {

        }
    }
}