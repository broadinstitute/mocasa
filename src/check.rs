use crate::error::Error;
use crate::options::config::Config;
use crate::params::Params;

pub(crate) fn check_config(config: &Config) -> Result<(), Error> {
    if config.gwas.is_empty() {
        return Err(Error::from("No GWAS specified."))
    }
    Ok(())
}

pub(crate) fn check_params(config: &Config, params: &Params) -> Result<(), Error> {
    if config.gwas.len() != params.trait_names.len() {
        return Err(Error::from(format!(
            "Number GWAS files ({}) does not match number of traits in params ({})",
            config.gwas.len(), params.trait_names.len())
        ))
    } else {
        for (i_trait, (gwas, trait_name)) in
        config.gwas.iter().zip(params.trait_names.iter()).enumerate() {
            if gwas.name != *trait_name {
                return Err(Error::from(format!(
                    "Trait name in GWAS file {} ({}) does not match trait name in params ({})",
                    i_trait, gwas.name, trait_name)
                ))
            }
        }
    }
    Ok(())
}