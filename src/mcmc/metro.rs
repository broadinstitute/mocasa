use std::cmp::Ordering;
use rand::Rng;

pub(crate) struct MetroHast<R: Rng> {
    rng: R,
}

pub(crate) struct Draw {
    pub(crate) x: f64,
    pub(crate) attempts_minus: usize,
    pub(crate) attempts_plus: usize,
}

impl<R: Rng> MetroHast<R> {
    pub(crate) fn new(rng: R) -> MetroHast<R> { MetroHast { rng } }
    pub(crate) fn draw<F: Fn(f64, f64) -> f64>(&mut self, f_quot: F, x_old: f64, sigma: f64)
                                               -> Draw {
        let mut attempts_minus: usize = 0;
        let mut attempts_plus: usize = 0;
        let x =
            loop {
                let x_diff = self.rng.gen_range(-sigma..sigma);
                match x_diff.total_cmp(&0.0) {
                    Ordering::Less => { attempts_minus += 1 }
                    Ordering::Equal => { /* do nothing */ }
                    Ordering::Greater => { attempts_plus += 1 }
                }
                let x_new = x_old + x_diff;
                let quot = f_quot(x_new, x_old);
                if quot >= 1.0 || quot > self.rng.gen_range(0.0..1.0) {
                    break x_new;
                }
            };
        Draw { x, attempts_minus, attempts_plus }
    }
}

