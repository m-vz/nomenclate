use std::path::Path;

use error::Error;
use lopdf::Document;

pub mod error;

/// Load a PDF document from a specific path.
///
/// # Errors
///
/// This function will return an error if the document could not be loaded or if it is encrypted.
pub fn load_document<P: AsRef<Path>>(path: P) -> Result<Document, Error> {
    Document::load(&path)
        .map_err(|err| Error::Load {
            path: path.as_ref().to_path_buf(),
            source: err,
        })
        .and_then(|doc| {
            if doc.is_encrypted() {
                Err(Error::Encrypted(path.as_ref().to_path_buf()))
            } else {
                Ok(doc)
            }
        })
}
