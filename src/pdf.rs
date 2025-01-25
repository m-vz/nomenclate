use std::{fmt::Display, path::Path};

use approx::{abs_diff_eq, abs_diff_ne};
use error::Error;
use font::{FontCache, FontInfo};
use pdf::{
    content::{Op, TextDrawAdjusted},
    file::FileOptions,
    object::{PageRc, Resolve},
    primitive::PdfString,
};

pub mod error;
mod font;

struct PositionedText {
    text: String,
    font_size: f32,
    y: f32,
}

impl PositionedText {
    fn from_text(text: &PdfString, state: &TextState) -> Self {
        Self {
            text: state.font.decode(text).expect("could not parse pdf string"),
            font_size: state.font_size,
            y: state.y,
        }
    }
    fn from_text_array(array: &[TextDrawAdjusted], state: &TextState) -> Self {
        Self {
            text: array
                .iter()
                .filter_map(|elem| match elem {
                    TextDrawAdjusted::Text(text) => {
                        Some(state.font.decode(text).expect("could not parse pdf string"))
                    }
                    TextDrawAdjusted::Spacing(spacing) => {
                        if *spacing < -100. {
                            Some(String::from(" "))
                        } else {
                            None
                        }
                    }
                })
                .collect::<String>(),
            font_size: state.font_size,
            y: state.y,
        }
    }
}

impl Display for PositionedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "text at y = {} with font size {}: {:?}",
            self.y, self.font_size, self.text
        )
    }
}

#[derive(Clone, Default)]
pub struct TextState {
    pub font: FontInfo,
    pub font_size: f32,
    pub leading: f32,
    pub y: f32,
}

/// Load a PDF document and parse the first `page_count` pages.
///
/// If the document has less than `page_count` pages, all pages are parsed.
///
/// If a page could not be parsed properly, it is skipped and a warning is shown to the user.
///
/// # Errors
///
/// This function will return an error if the document could not be loaded.
pub fn parse_pdf<P: AsRef<Path>>(path: P, page_count: usize) -> Result<String, Error> {
    let path = path.as_ref().to_path_buf();
    let file = FileOptions::cached()
        .open(path.clone())
        .map_err(|err| Error::Load { path, source: err })?;
    let resolver = file.resolver();
    let mut max_font_size = 0.;
    let mut text = String::new();

    for (page_number, page) in
        file.pages()
            .take(page_count)
            .enumerate()
            .filter_map(|(page_number, page)| {
                page.inspect_err(|err| log::warn!("skipping page {page_number}: {err}"))
                    .map(|page| (page_number, page))
                    .ok()
            })
    {
        if let Ok((page_text, font_size)) = largest_text_elements(&page, &resolver)
            .inspect_err(|err| log::error!("could not parse page {page_number}: {err}"))
        {
            if font_size > max_font_size {
                text = page_text
                    .into_iter()
                    .map(|text| text.text)
                    .collect::<Vec<_>>()
                    .join(" ");
                max_font_size = font_size;
            }
        }
    }

    Ok(text)
}

fn largest_text_elements(
    page: &PageRc,
    resolver: &impl Resolve,
) -> Result<(Vec<PositionedText>, f32), Error> {
    let font_cache = FontCache::from_page(page, resolver);
    let mut state = TextState::default();
    let mut max_font_size = 0.;
    let mut positioned_text = Vec::new();

    for operation in page
        .contents
        .as_ref()
        .ok_or(Error::NoContent)?
        .operations(resolver)?
    {
        match operation {
            Op::BeginText => {
                log::debug!("reset text state");
                state.font_size = 0.;
                state.leading = 0.;
                state.y = 0.;
            }
            Op::Leading { leading: amount } => {
                log::debug!("leading: {amount}");
                state.leading = amount;
            }
            Op::GraphicsState { ref name } => {
                if let Some((font, size)) =
                    font_cache.get_font_from_graphic_state(name, page, resolver)
                {
                    log::debug!("graphics state font {name} ({size})");
                    state.font = font;
                    state.font_size = size;

                    if size > max_font_size {
                        max_font_size = size;
                    }
                }
            }
            Op::TextFont { ref name, size } => {
                log::debug!("font {name} ({size})");
                state.font = font_cache.get_font(name);
                state.font_size = size;

                if size > max_font_size {
                    max_font_size = size;
                }
            }
            // `Td`, `TD`
            Op::MoveTextPosition { translation } => {
                translate_text(&mut state, translation.y);
            }
            // `Tm`
            Op::SetTextMatrix { matrix } => {
                state.y = matrix.f;
                log::debug!("set y = {}", state.y);
            }
            // `T*`
            Op::TextNewline => {
                let dy = -state.leading;
                translate_text(&mut state, dy);
            }
            // `Tj`
            Op::TextDraw { text } => {
                let text = PositionedText::from_text(&text, &state);
                log::debug!("write {text}");
                positioned_text.push(text);
            }
            Op::TextDrawAdjusted { array } => {
                let text = PositionedText::from_text_array(&array, &state);
                log::debug!("write {text}");
                positioned_text.push(PositionedText::from_text_array(&array, &state));
            }
            operation => log::trace!("skipping operation {operation:?}"),
        }
    }

    log::info!("max font size: {max_font_size}");
    Ok((
        positioned_text
            .into_iter()
            .filter(|text| abs_diff_eq!(text.font_size, max_font_size))
            .collect(),
        max_font_size,
    ))
}

fn translate_text(state: &mut TextState, dy: f32) {
    if abs_diff_ne!(dy, 0.) {
        state.y += dy;
        log::debug!("translate y by {dy}, y = {}", state.y);
    }
}
