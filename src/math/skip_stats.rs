#[derive(Clone)]
pub(crate) struct SkipStats {
    pub(crate) n: usize,
    pub(crate) sum: f64,
    pub(crate) var_sum: f64,
}

impl SkipStats {
    pub(crate) fn new(x0: f64, x1: f64) -> SkipStats {
        let n: usize = 2;
        let sum = x0 + x1;
        let var_sum = 0.5 * (x1 - x0).powi(2);
        SkipStats { n, sum, var_sum }
    }
    pub(crate) fn add(&mut self, value: f64) {
        let mean_previous = self.sum / (self.n as f64);
        self.n += 1;
        self.sum += value;
        let mean = self.sum / (self.n as f64);
        self.var_sum += (value - mean_previous) * (value - mean)  //  Welford's method
    }
}
