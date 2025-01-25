use std::collections::HashMap;

use pdf::{
    encoding::BaseEncoding,
    font::{self, Font, ToUnicodeMap},
    object::{Page, RcRef, Resolve},
    primitive::{Name, PdfString},
    PdfError,
};
use pdf_encoding::DifferenceForwardMap;

use super::error::Error;

#[derive(Clone, Default)]
enum Decoder {
    Map(DifferenceForwardMap),
    Cmap(ToUnicodeMap),
    #[default]
    None,
}

impl Decoder {
    fn from_font(font: &Font, resolver: &impl Resolve) -> Result<Self, Error> {
        if let Some(Ok(to_unicode)) = font.to_unicode(resolver) {
            Ok(Self::Cmap(to_unicode))
        } else if let Some(encoding) = font.encoding() {
            Ok(Self::Map(DifferenceForwardMap::new(
                match &encoding.base {
                    BaseEncoding::StandardEncoding => Some(&pdf_encoding::STANDARD),
                    BaseEncoding::SymbolEncoding => Some(&pdf_encoding::SYMBOL),
                    BaseEncoding::WinAnsiEncoding => Some(&pdf_encoding::WINANSI),
                    BaseEncoding::MacRomanEncoding => Some(&pdf_encoding::MACROMAN),
                    BaseEncoding::None => None,
                    other => {
                        return Err(Error::UnsupportedEncoding(other.clone()));
                    }
                },
                encoding
                    .differences
                    .iter()
                    .map(|(k, v)| (*k, v.to_string()))
                    .collect(),
            )))
        } else {
            Err(Error::MissingEncoding(
                font.name
                    .clone()
                    .unwrap_or_else(|| Name::from("MISSING_NAME")),
            ))
        }
    }
}

#[derive(Default, Clone)]
pub struct FontInfo(Decoder);

impl FontInfo {
    pub fn decode(&self, text: &PdfString) -> Result<String, Error> {
        let data = &text.data;

        match &self.0 {
            Decoder::Map(map) => Ok(data
                .iter()
                .filter_map(|&b| map.get(b))
                .cloned()
                .collect::<String>()),
            Decoder::Cmap(ref cmap) => {
                // TODO: check for BOMs other than UTF-16BE
                if data.starts_with(&[0xfe, 0xff]) {
                    let mut text = String::new();
                    data.chunks_exact(2)
                        .map(|chunk| {
                            u16::from_be_bytes(chunk.try_into().unwrap_or_else(|_| {
                                unreachable!(
                                    "This cannot fail because we use exact chunks of length 2."
                                )
                            }))
                        })
                        .filter_map(|code| cmap.get(code))
                        .for_each(|mapped| text.push_str(mapped));
                    Ok(text)
                } else {
                    Ok(data
                        .iter()
                        .filter_map(|&b| cmap.get(b.into()))
                        .collect::<String>())
                }
            }
            Decoder::None => {
                // TODO: check for BOMs other than UTF-16BE
                if data.starts_with(&[0xfe, 0xff]) {
                    let mut text = String::new();
                    font::utf16be_to_char(&data[2..]).try_for_each(|result| {
                        result.map_or(Err(Error::Pdf(PdfError::Utf16Decode)), |c| {
                            text.push(c);
                            Ok(())
                        })
                    })?;
                    Ok(text)
                } else if let Ok(text) = std::str::from_utf8(data) {
                    Ok(text.to_string())
                } else {
                    Err(Error::Pdf(PdfError::Utf16Decode))
                }
            }
        }
    }
}

pub struct FontCache(HashMap<Name, FontInfo>);

impl FontCache {
    pub fn from_page(page: &Page, resolver: &impl Resolve) -> Self {
        let mut font_cache = Self(HashMap::new());

        if let Ok(resources) = page.resources() {
            for (name, font) in &resources.fonts {
                if let Some(font) = font.as_ref() {
                    if let Ok(font) = resolver.get(font) {
                        font_cache.add_font(name, &font, resolver);
                    }
                }
            }

            for (font, _) in resources
                .graphics_states
                .values()
                .filter_map(|state| state.font)
            {
                if let Ok(font) = resolver.get(font) {
                    if let Some(name) = &font.name {
                        font_cache.add_font(name, &font, resolver);
                    }
                }
            }
        }

        font_cache
    }

    pub fn get_font(&self, name: &Name) -> FontInfo {
        self.0.get(name).cloned().unwrap_or_else(FontInfo::default)
    }

    pub fn get_font_from_graphic_state(
        &self,
        name: &Name,
        page: &Page,
        resolver: &impl Resolve,
    ) -> Option<(FontInfo, f32)> {
        page.resources()
            .ok()
            .and_then(|resources| resources.graphics_states.get(name))
            .and_then(|state| state.font)
            .map(|(font, font_size)| {
                (
                    resolver
                        .get(font)
                        .ok()
                        .and_then(|font| Some(self.get_font(font.name.as_ref()?)))
                        .unwrap_or_else(FontInfo::default),
                    font_size,
                )
            })
    }

    fn add_font(&mut self, name: &Name, font: &RcRef<Font>, resolver: &impl Resolve) {
        let _ = Decoder::from_font(font, resolver)
            .inspect_err(|err| log::warn!("Unable to add font: {err}"))
            .map(FontInfo)
            .map(|font_info| self.0.insert(name.clone(), font_info));
    }
}
