use std::{fmt::Display, path::PathBuf};

use colored::*;
use indenter::indented;

#[derive(Debug, thiserror::Error)]
pub struct Error {
    pub file_path: PathBuf,
    pub line_num: usize,
    pub error_type: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.file_path.to_str().unwrap(),
            self.line_num,
            self.error_type
        )
    }
}

#[derive(Debug, strum_macros::Display)]
pub enum ErrorKind {
    #[strum(serialize = "unterminated comment")]
    UnterminatedComment,
}
