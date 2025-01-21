use std::fmt::{Debug, Display};

use lopdf::{content, Object};
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
}

impl TryFrom<content::Operation> for Operation {
    type Error = Error;

    fn try_from(
        content::Operation { operator, operands }: content::Operation,
    ) -> Result<Self, Self::Error> {
        let parse_with_param = |param_index, operation: fn(f32) -> Self| {
            operands
                .get(param_index)
                .and_then(|param: &Object| param.as_float().ok())
                .map(operation)
                .ok_or_else(|| Error::ParseError(operator.clone()))
        };

        match operator.as_str() {
            // BT
            "BT" => Ok(Self::BeginText),
            // ET
            "ET" => Ok(Self::EndText),
            // leading TL
            "TL" => parse_with_param(0, Self::Leading),
            // font size Tf
            "Tf" => parse_with_param(1, Self::FontSize),
            // tx ty Td
            "Td" => parse_with_param(1, Self::Offset),
            // tx ty TD
            "TD" => parse_with_param(1, Self::OffsetWithLeading),
            // a b c d e f Tm
            "Tm" => parse_with_param(5, Self::Position),
            // T*
            "T*" => Ok(Self::OffsetByLeading),
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
        }
    }
}
