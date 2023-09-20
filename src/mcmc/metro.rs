use rand::Rng;

pub(crate) struct MetroHast<R: Rng> {
    rng: R,
}

pub(crate) struct Draw {
    x: f64,
    attempts: usize,
}

impl<R: Rng> MetroHast<R> {
    pub(crate) fn new(rng: R) -> MetroHast<R> { MetroHast { rng } }
    pub(crate) fn draw<F: Fn(f64, f64) -> f64>(&mut self, f_quot: F, x0: f64, sigma: f64) -> Draw {
        let mut attempts: usize = 0;
        let x =
            loop {
                attempts += 1;
                let x = self.rng.gen_range((x0 - sigma)..(x0 + sigma));
                let quot = f_quot(x0, x);
                if quot >= 1.0 || quot > self.rng.gen_range(0.0..1.0) {
                    break x;
                }
            };
        Draw { x, attempts }
    }
}

