#[derive(Clone)]
pub(crate) struct Stats {
    pub(crate) n: usize,
    pub(crate) sum: f64,
    pub(crate) var_sum: f64,
}

impl Stats {
    pub(crate) fn new() -> Stats {
        let n: usize = 0;
        let sum: f64 = 0.0;
        let var_sum: f64 = 0.0;
        Stats { n, sum, var_sum }
    }
    pub(crate) fn add(&mut self, value: f64) {
        if self.n == 0 {
            self.n += 1;
            self.sum += value;
        } else {
            let mean_previous = self.sum / (self.n as f64);
            self.n += 1;
            self.sum += value;
            let mean = self.sum / (self.n as f64);
            self.var_sum += (value - mean_previous) * (value - mean)  //  Welford's method
        }
    }
    pub(crate) fn mean(&self) -> Option<f64> {
        if self.n == 0 {
            None
        } else {
            Some(self.sum / (self.n as f64))
        }
    }
    pub(crate) fn variance(&self) -> Option<f64> {
        if self.n < 2 {
            None
        } else {
            Some(self.var_sum / (self.n as f64))
        }
    }
}