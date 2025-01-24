use std::fmt::{Debug, Display};

use lopdf::{content, Object, StringFormat};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("the operation {0} is not implemented")]
    NotImplemented(String),
    #[error("the operation {0} could not be parsed")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub enum Operation {
    BeginText,
    EndText,
    Leading(f32),
    FontSize(f32),
    Offset(f32),
    OffsetWithLeading(f32),
    Position(f32),
    OffsetByLeading,
    ShowText(String),
    ShowTextWithOffsetByLeading(String),
}

impl TryFrom<content::Operation> for Operation {
    type Error = Error;

    fn try_from(
        content::Operation { operator, operands }: content::Operation,
    ) -> Result<Self, Self::Error> {
        let parse_number = |param_index, operation: fn(f32) -> Self| {
            operands
                .get(param_index)
                .and_then(|param: &Object| param.as_float().ok())
                .map(operation)
                .ok_or_else(|| Error::ParseError(operator.clone()))
        };
        let parse_text = |param_index, operation: fn(String) -> Self| {
            operands
                .get(param_index)
                .and_then(|param: &Object| object_to_string(param))
                .map(|_| String::new())
                .map(operation)
                .ok_or_else(|| Error::ParseError(operator.clone()))
        };

        match operator.as_str() {
            // BT
            "BT" => Ok(Self::BeginText),
            // ET
            "ET" => Ok(Self::EndText),
            // leading TL
            "TL" => parse_number(0, Self::Leading),
            // font size Tf
            "Tf" => parse_number(1, Self::FontSize),
            // tx ty Td
            "Td" => parse_number(1, Self::Offset),
            // tx ty TD
            "TD" => parse_number(1, Self::OffsetWithLeading),
            // a b c d e f Tm
            "Tm" => parse_number(5, Self::Position),
            // T*
            "T*" => Ok(Self::OffsetByLeading),
            // string Tj
            "Tj" => parse_text(0, Self::ShowText),
            // string ' | string "
            "'" | "\"" => parse_text(0, Self::ShowTextWithOffsetByLeading),
            // [string spacing] TJ
            "TJ" => Ok(Self::ShowText(
                operands
                    .iter()
                    .filter_map(|params| params.as_array().ok())
                    .flatten()
                    .filter_map(object_to_string)
                    .fold(String::new(), |acc, x| acc + &x),
            )),
            _ => Err(Error::NotImplemented(operator)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeginText => write!(f, "begin text"),
            Self::EndText => write!(f, "end text"),
            Self::Leading(leading) => write!(f, "  set leading {leading}"),
            Self::FontSize(size) => write!(f, "  font size {size}"),
            Self::Offset(offset) => write!(f, "  offset y by {offset}"),
            Self::OffsetWithLeading(offset) => Display::fmt(&Self::Leading(-offset), f)
                .and_then(|()| Display::fmt(&Self::Offset(*offset), f)),
            Self::Position(position) => write!(f, "  set y to {position}"),
            Self::OffsetByLeading => write!(f, "  offset y by current leading"),
            Self::ShowText(text) => write!(f, "  write \"{text}\""),
            Self::ShowTextWithOffsetByLeading(text) => Display::fmt(&Self::OffsetByLeading, f)
                .and_then(|()| Display::fmt(&Self::ShowText(text.to_owned()), f)),
        }
    }
}

fn object_to_string(object: &Object) -> Option<String> {
    let text = match object {
        Object::String(text, StringFormat::Literal) => text,
        Object::String(text, StringFormat::Hexadecimal) => &text
            .chunks(2)
            .filter_map(|chunk| {
                let hex = String::from_utf8_lossy(chunk);
                let hex = if chunk.len() == 1 {
                    format!("{hex}0").into()
                } else {
                    hex
                };
                u8::from_str_radix(&hex, 16).ok()
            })
            .collect::<Vec<_>>(),
        Object::Integer(i) => {
            if *i < -100 {
                return Some(' '.to_string());
            }
            return None;
        }
        _ => return None,
    };

    Some(String::from_utf8_lossy(text).into_owned())
}
