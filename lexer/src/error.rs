use std::path::{Path, PathBuf};

use crate::token::Position;

#[derive(Debug, thiserror::Error)]
#[error("{}:{}: {}", .pos.line, .pos.col, .error_kind)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Error {
    pub pos: Position,
    pub error_kind: ErrorKind,
}

#[derive(thiserror::Error, Debug)]
pub enum LexError {
    #[error("failed to preprocess {}:{}:{}", Path::to_str(file_path).unwrap(), source.pos.line, source.pos.col)]
    PreprocessError { file_path: PathBuf, source: Error },
    #[error("failed to scan {}", Path::to_str(file_path).unwrap())]
    TokenError { file_path: PathBuf, source: Error },
}

#[derive(Debug, strum_macros::Display)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ErrorKind {
    #[strum(serialize = "unterminated comment")]
    UnterminatedComment,
    #[strum(serialize = "expect digit after \'.\'")]
    ExpectDigit,
}
