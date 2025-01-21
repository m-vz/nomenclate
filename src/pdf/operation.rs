use std::fmt::Display;

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
