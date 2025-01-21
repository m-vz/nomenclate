use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not load document: {path}")]
    Load {
        path: PathBuf,
        #[source]
        source: lopdf::Error,
    },
    #[error("document is encrypted: {0}")]
    Encrypted(PathBuf),
}
