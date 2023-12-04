use std::path::Path;
use crate::error::Error;

pub(crate) fn check_dir_exists(path: &str) -> Result<(), Error> {
    Path::new(path).parent().iter().for_each(|parent| {
        if !parent.exists() {
            Err(Error::from(format!("File {} does not exist", parent.display())))?
        }
    });
    Ok(())
}