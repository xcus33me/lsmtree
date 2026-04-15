use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// #[derive(Debug)]
// #[non_exhaustive]
// pub enum WalError {
//     Write(std::io::Error),
//     RecordTooLarge { size: usize, max: usize },
// }

// impl Display for WalError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             WalError::Write(e) => write!(f, "WAL write error: {e}"),
//             WalError::RecordTooLarge { size, max } => {
//                 write!(f, "record too large: {size} bytes, max {max} bytes")
//             }
//         }
//     }
// }

// impl std::error::Error for WalError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             WalError::Write(e) => Some(e),
//             _ => None,
//         }
//     }
// }

// impl From<std::io::Error> for WalError {
//     fn from(value: std::io::Error) -> Self {
//         WalError::Write(value)
//     }
// }
