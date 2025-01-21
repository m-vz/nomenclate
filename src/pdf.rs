use std::path::{Path, PathBuf};

use error::Error;
use lopdf::{content::Content, Document, ObjectId};
use operation::Operation;

pub mod error;
mod operation;

pub struct Parser {
    path: PathBuf,
    document: Document,
    font_size: Option<f32>,
}

impl Parser {
    /// Load a PDF document from a specific path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the document could not be loaded or is encrypted.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let document = Document::load(&path)
            .map_err(|err| Error::Load {
                path: path.clone(),
                source: err,
            })
            .and_then(|doc| {
                if doc.is_encrypted() {
                    Err(Error::Encrypted(path.clone()))
                } else {
                    Ok(doc)
                }
            })?;

        Ok(Self {
            path,
            document,
            font_size: None,
        })
    }

    /// Parse the first `page_count` pages of a document.
    ///
    /// If the document has less than `page_count` pages, all pages are parsed and a warning is
    /// shown to the user.
    ///
    /// If a page could not be parsed properly, it is skipped and a warning is shown to the user.
    pub fn parse_pages(&mut self, page_count: u32) {
        for page in 1..=page_count {
            if let Some(page_id) = self.page_id(page) {
                log::debug!("Parsing page {page}");

                if let Err(err) = self.parse_page(page_id) {
                    log::error!(
                        "An error occurred when parsing page {page} of {:?}: {err}",
                        self.path
                    );
                }
            } else {
                log::warn!("Document {:?} did not have enough pages", self.path);
            }
        }
    }

    fn parse_page(&mut self, page: ObjectId) -> Result<(), Error> {
        let content = Content::decode(&self.document.get_page_content(page).map_err(Error::Pdf)?)
            .map_err(Error::Pdf)?;

        content
            .operations
            .into_iter()
            .flat_map(Operation::try_from)
            .for_each(|operation| {
                log::trace!("{operation}");

                match operation {
                    Operation::FontSize(size) => {
                        self.font_size = Some(size);
                    }
                    _ => {}
                }
            });

        Ok(())
    }

    fn page_id(&self, page_number: u32) -> Option<ObjectId> {
        self.document.get_pages().get(&page_number).copied()
    }
}
