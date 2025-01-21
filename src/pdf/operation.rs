use std::fmt::Display;

use lopdf::content;
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
}

impl TryFrom<content::Operation> for Operation {
    type Error = Error;

    fn try_from(
        content::Operation { operator, operands }: content::Operation,
    ) -> Result<Self, Self::Error> {
        match operator.as_str() {
            // BT
            "BT" => Ok(Self::BeginText),
            // ET
            "ET" => Ok(Self::EndText),
            // leading TL
            "TL" => operands
                .first()
                .and_then(|leading| leading.as_float().ok())
                .map(Self::Leading)
                .ok_or(Error::ParseError(operator)),
            // font size Tf
            "Tf" => operands
                .get(1)
                .and_then(|size| size.as_float().ok())
                .map(Self::FontSize)
                .ok_or(Error::ParseError(operator)),
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
        }
    }
}
