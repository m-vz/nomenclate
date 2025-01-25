use std::path::PathBuf;

use pdf::{encoding::BaseEncoding, primitive::Name, PdfError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not load document: {path}")]
    Load {
        path: PathBuf,
        #[source]
        source: PdfError,
    },
    #[error("page has no content")]
    NoContent,
    #[error("unsupported encoding: {0:?}")]
    UnsupportedEncoding(BaseEncoding),
    #[error("font {0:?} is missing an encoding")]
    MissingEncoding(Name),
    #[error("an error occurred when parsing the pdf: {0}")]
    Pdf(#[from] PdfError),
}
