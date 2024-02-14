use std::path::Path;
use crate::error::Error;

pub(crate) fn check_parent_dir_exists(path: &str) -> Result<(), Error> {
    if let Some(parent) = Path::new(path).parent() {
        if parent != Path::new("") && !parent.exists() {
            Err(Error::from(format!("File {} does not exist", parent.display())))?
        }
    };
    Ok(())
}