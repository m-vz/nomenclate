use std::path::{Path, PathBuf};

use error::Error;
use lopdf::Document;

pub mod error;

pub struct Parser {
    path: PathBuf,
    document: Document,
}

impl Parser {
    /// Load a PDF document from a specific path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the document could not be loaded or is encrypted.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();

        Ok(Self {
            path: path.clone(),
            document: Document::load(&path)
                .map_err(|err| Error::Load {
                    path: path.clone(),
                    source: err,
                })
                .and_then(|doc| {
                    if doc.is_encrypted() {
                        Err(Error::Encrypted(path))
                    } else {
                        Ok(doc)
                    }
                })?,
        })
    }
}
