use std::error::Error;
use std::fmt::{Display, Formatter};
use nuts_rs::{CpuLogpFunc, LogpError, new_sampler, SamplerArgs, Chain, SampleStats};
use thiserror::Error;

// Define a function that computes the unnormalized posterior density
// and its gradient.
struct PosteriorDensity {}

// The density might fail in a recoverable or non-recoverable manner...
#[derive(Debug, Error)]
enum PosteriorLogpError {}

impl Error for PosteriorLogpError {}

impl Display for PosteriorLogpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl LogpError for PosteriorLogpError {
    fn is_recoverable(&self) -> bool { false }
}

impl CpuLogpFunc for PosteriorDensity {
    type Err = PosteriorLogpError;

    // The normal likelihood with mean 3 and its gradient.
    fn logp(&mut self, position: &[f64], grad: &mut [f64]) -> Result<f64, Self::Err> {
        let mu = 3f64;
        let logp = position
            .iter()
            .copied()
            .zip(grad.iter_mut())
            .map(|(x, grad)| {
                let diff = x - mu;
                *grad = -diff;
                -diff * diff / 2f64
            })
            .sum();
        return Ok(logp);
    }

    // We define a 10 dimensional normal distribution
    fn dim(&self) -> usize { 10 }
}

pub fn run_example() {

// We get the default sampler arguments
    let mut sampler_args = SamplerArgs::default();

// and modify as we like
    sampler_args.num_tune = 1000;
    sampler_args.maxdepth = 3;  // small value just for testing...

// We instanciate our posterior density function
    let logp_func = PosteriorDensity {};

    let chain = 0;
    let seed = 42;
    let mut sampler = new_sampler(logp_func, sampler_args, chain, seed);

// Set to some initial position and start drawing samples.
    sampler.set_position(&vec![0f64; 10]).expect("Unrecoverable error during init");
    let mut trace = vec![];  // Collection of all draws
    let mut stats = vec![];
    // Collection of statistics like the acceptance rate for each draw
    for _ in 0..2000 {
        let (draw, info) =
            sampler.draw().expect("Unrecoverable error during sampling");
        trace.push(draw);
        let _info_vec = info.to_vec();  // We can collect the stats in a Vec
// Or get more detailed information about divergences
        if let Some(div_info) = info.divergence_info() {
            println!("Divergence at position {:?}", div_info.start_location());
        }
        dbg!(&info);
        stats.push(info);
    }
}
