use std::collections::VecDeque;

#[derive(Clone)]
struct Tally {
    n: u64,
    mean: f64,
    m2: f64,
}
pub(crate) struct SnappedStats {
    tally: Tally,
    n_between_snaps: u64,
    snaps: VecDeque<Tally>,
}

impl Tally {
    pub(crate) fn new() -> Tally {
        let n: u64 = 0;
        let mean: f64 = 0.0;
        let m2: f64 = 0.0;
        Tally { n, mean, m2 }
    }
    pub(crate) fn add(&mut self, value: f64) {
        self.n += 1;
        let delta = value - self.mean;
        self.mean += delta / (self.n as f64);
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }
    pub(crate) fn mean(&self) -> f64 { self.mean }
    pub(crate) fn variance(&self) -> f64 { self.m2 / (self.n as f64 - 1.0) }
    pub(crate) fn plus(&self, rhs: &Tally) -> Tally {
        let n = self.n + rhs.n;
        let n_s_f = self.n as f64;
        let n_r_f = rhs.n as f64;
        let n_f = n as f64;
        let mean = (n_s_f * self.mean + n_r_f * rhs.mean) / n_f;
        let m2 = self.m2 + rhs.m2 + (self.mean - rhs.mean).powi(2) * n_s_f * n_r_f / n_f;
        Tally { n, mean, m2 }
    }
    pub(crate) fn minus(&self, rhs: &Tally) -> Tally {
        let n = self.n - rhs.n;
        let n_s_f = self.n as f64;
        let n_r_f = rhs.n as f64;
        let n_f = n as f64;
        let mean = (n_s_f * self.mean - n_r_f * rhs.mean) / n_f;
        let m2 = self.m2 - rhs.m2 - (rhs.mean - mean).powi(2) * n_f * n_r_f / n_s_f;
        Tally { n, mean, m2 }
    }
}
impl SnappedStats {
    pub(crate) fn new() -> SnappedStats {
        let tally = Tally::new();
        let n_between_snaps: u64 = 4;
        let snaps: VecDeque<Tally> = VecDeque::new();
        SnappedStats { tally, n_between_snaps, snaps }
    }
    pub(crate) fn add(&mut self, value: f64) {
        self.tally.add(value);
        if self.tally.n % self.n_between_snaps == 0 {
            self.snaps.push_back(self.tally.clone());
        }
    }
    pub(crate) fn mean(&self) -> f64 { self.tally.mean() }
    pub(crate) fn variance(&self) -> f64 { self.tally.variance() }
    pub(crate) fn reduce_snaps(&mut self) {
        let mut index: u64 = 0;
        self.snaps.retain(|_| {
            let is_odd = index % 2 == 1;
            index += 1;
            is_odd
        });
        self.n_between_snaps *= 2;
    }
    pub(crate) fn drop_before_first_snap(&mut self) {
        if let Some(first_snap) = self.snaps.pop_front() {
            self.tally = self.tally.minus(&first_snap);
        }
    }
}