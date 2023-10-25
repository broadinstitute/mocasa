use crate::error::Error;

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
    pub(crate) fn mean(&self) -> f64 { self.sum / (self.n as f64) }
    pub(crate) fn variance(&self) -> f64 { self.var_sum / (self.n as f64) }
    pub(crate) fn try_minus(&self, rhs: &SkipStats) -> Result<SkipStats, Error> {
        let n = self.n.checked_sub(rhs.n).ok_or_else(n_underflow_error)?;
        if n < 2 {
            Err(n_underflow_error())?;
        }
        let sum = self.sum - rhs.sum;
        let self_square_sum = self.var_sum + (self.n as f64) * self.mean().powi(2);
        let rhs_square_sum = rhs.var_sum + (rhs.n as f64) * rhs.mean().powi(2);
        let square_sum_new = self_square_sum - rhs_square_sum;
        let var_sum = square_sum_new - sum.powi(2) / (n as f64);
        if var_sum < 0.0 {
            Err(Error::from("var_sum needs to be non-negative."))?
        }
        Ok(SkipStats { n, sum, var_sum })
    }
}

fn n_underflow_error() -> Error {
    Error::from("n must be at least 2.")
}