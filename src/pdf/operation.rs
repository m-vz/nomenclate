use std::fmt::Display;

use lopdf::content;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("the operation {0} is not implemented")]
    NotImplemented(String),
}

#[derive(Debug, Clone)]
pub enum Operation {
    FontSize(f32),
    BeginText,
    EndText,
}

impl TryFrom<content::Operation> for Operation {
    type Error = Error;

    fn try_from(
        content::Operation { operator, operands }: content::Operation,
    ) -> Result<Self, Self::Error> {
        match operator.as_str() {
            // Tf font size
            "Tf" => operands
                .get(1)
                .and_then(|size| size.as_f32().ok())
                .map(Self::FontSize)
                .ok_or(Error::NotImplemented(operator)),
            // BT
            "BT" => Ok(Self::BeginText),
            // ET
            "ET" => Ok(Self::EndText),
            _ => Err(Error::NotImplemented(operator)),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FontSize(size) => write!(f, "    font size {size}"),
            Self::BeginText => write!(f, "  begin text"),
            Self::EndText => write!(f, "  end text"),
        }
    }
}
