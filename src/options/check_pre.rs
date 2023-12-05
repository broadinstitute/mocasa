use crate::error::Error;
use crate::options::config::Config;
use crate::util::files::check_parent_dir_exists;

pub(crate) fn check_prerequisites(config: &Config) -> Result<(), Error> {
    if let Some(trace_file) = &config.files.trace {
        check_parent_dir_exists(trace_file)?
    }
    check_parent_dir_exists(&config.files.params)?;
    Ok(())
}