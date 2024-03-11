use crate::error::Error;
use crate::options::cli::ScaleSigmasOptions;
use crate::params::{Params, read_params_from_file, write_params_to_file};

pub(crate) fn scale_sigmas(config: &ScaleSigmasOptions) -> Result<(), Error> {
    let Params { trait_names, mu, tau, betas, sigmas }
        = read_params_from_file(&config.in_file)?;
    let sigmas: Vec<f64> = sigmas.iter().map(|sigma| sigma * config.scale).collect();
    let params = Params { trait_names, mu, tau, betas, sigmas };
    write_params_to_file(&params, &config.out_file)?;
    Ok(())
}