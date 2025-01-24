use std::path::Path;

use error::Error;
use pdf::{
    content::{Op, TextDrawAdjusted},
    file::FileOptions,
    object::{PageRc, Resolve},
};

pub mod error;

/// Load a PDF document and parse the first `page_count` pages.
///
/// If the document has less than `page_count` pages, all pages are parsed.
///
/// If a page could not be parsed properly, it is skipped and a warning is shown to the user.
///
/// # Errors
///
/// This function will return an error if the document could not be loaded.
pub fn parse_pdf<P: AsRef<Path>>(path: P, page_count: usize) -> Result<(), Error> {
    let path = path.as_ref().to_path_buf();
    let file = FileOptions::cached()
        .open(path.clone())
        .map_err(|err| Error::Load { path, source: err })?;
    let resolver = file.resolver();

    file.pages()
        .take(page_count)
        .enumerate()
        .filter_map(|(page_number, page)| {
            page.inspect_err(|err| log::warn!("skipping page {page_number}: {err}"))
                .map(|page| (page_number, page))
                .ok()
        })
        .for_each(|(page_number, page)| {
            if let Err(err) = parse_page(&page, &resolver) {
                log::error!("could not parse page {page_number}: {err}");
            }
        });

    Ok(())
}

fn parse_page(page: &PageRc, resolver: &impl Resolve) -> Result<(), Error> {
    let mut font_size = 0.;
    let mut leading = 0.;
    let mut y = 0.;

    for operation in page
        .contents
        .as_ref()
        .ok_or(Error::NoContent)?
        .operations(resolver)?
    {
        match operation {
            Op::BeginText => {
                log::debug!("reset text state");
                font_size = 0.;
                leading = 0.;
                y = 0.;
            }
            Op::Leading { leading: amount } => {
                log::debug!("leading: {amount}");
                leading = amount;
            }
            Op::TextFont { size, .. } => {
                log::debug!("font size: {size}");
                font_size = size;
            }
            // `Td`, `TD`
            Op::MoveTextPosition { translation } => {
                translate_text(&mut y, translation.y);
            }
            // `Tm`
            Op::SetTextMatrix { matrix } => {
                y = matrix.f;
                log::debug!("set y = {y}");
            }
            // `T*`
            Op::TextNewline => {
                translate_text(&mut y, -leading);
            }
            // `Tj`
            Op::TextDraw { text } => log::info!(
                "write {:?}",
                text.to_string().expect("could not parse string")
            ),
            Op::TextDrawAdjusted { array } => {
                log::info!(
                    "write adjusted {:?}",
                    array
                        .iter()
                        .filter_map(|elem| match elem {
                            TextDrawAdjusted::Text(text) =>
                                Some(text.to_string().expect("could not parse string")),
                            TextDrawAdjusted::Spacing(spacing) =>
                                if *spacing < -100. {
                                    Some(String::from(" "))
                                } else {
                                    None
                                },
                        })
                        .collect::<String>()
                );
            }
            operation => log::trace!("skipping operation {operation:?}"),
        }
    }

    Ok(())
}

fn translate_text(y: &mut f32, dy: f32) {
    if abs_diff_ne!(dy, 0.) {
        *y += dy;
        log::debug!("translate y by {dy}, y = {y}");
    }
}
