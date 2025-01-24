use std::path::PathBuf;

use pdf::PdfError;
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
    #[error("an error occurred when parsing the pdf: {0}")]
    Pdf(#[from] PdfError),
}
