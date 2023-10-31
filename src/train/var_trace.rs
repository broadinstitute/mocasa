use crate::mcmc::metro::Draw;

pub(crate) struct VarTracer {
    pub(crate) mean: f64,
    pub(crate) variance: f64,
    pub(crate) rigidity: usize,
}

const RIGIDITY_INITIAL: usize = 3;
const RIGIDITY_MAX: usize = 100;

impl VarTracer {
    pub(crate) fn new(mean_estimate: f64, std_dev_estimate: f64) -> VarTracer {
        let mean = mean_estimate;
        let variance = std_dev_estimate.sqrt();
        let rigidity = RIGIDITY_INITIAL;
        VarTracer { mean, variance, rigidity }
    }
    pub(crate) fn add(&mut self, draw: Draw) {
        let x = draw.x;
        let p = 1.0 / (self.rigidity as f64);
        let q = 1.0 - p;
        self.mean = q * self.mean + p * x;
        self.variance = q * self.variance + p * (x - self.mean).powi(2);
        if self.rigidity < RIGIDITY_MAX {
            self.rigidity += 1;
        }
    }
    pub(crate) fn soften_stats(&mut self) { self.rigidity = RIGIDITY_INITIAL }
    pub(crate) fn std_dev(&self) -> f64 { self.variance.sqrt() }
}