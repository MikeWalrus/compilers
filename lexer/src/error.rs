use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::exit,
};

use crate::token::Position;
use anyhow::{Context, Result};
use colored::Colorize;

#[derive(Debug, thiserror::Error)]
#[error("{}:{}: {:?}", .pos.line, .pos.col, .error_kind)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Error {
    pub pos: Position,
    pub error_kind: ErrorKind,
}

impl Error {
    pub fn report(&self, file_path: &Path) -> Result<!> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        eprintln!("{}:{}", file_path.display(), self);
        let line = reader
            .lines()
            .nth(self.pos.line - 1)
            .with_context(|| format!("cannot find line {}", self.pos.line))??;
        let error_line = format!("{} | ", self.pos.line);
        let prefix_len = error_line.len();
        eprintln!("{}{}", error_line.blue().bold(), line);
        eprint!(
            "{:>width$}",
            "^".yellow().bold(),
            width = prefix_len + self.pos.col
        );
        eprintln!(" {}", self.error_kind.to_string().red().bold().italic());
        exit(1)
    }
}

#[derive(Debug, strum_macros::Display)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ErrorKind {
    #[strum(serialize = "unterminated comment")]
    UnterminatedComment,
    #[strum(serialize = "expect a digit before or after \'.\'")]
    ExpectDigit,
}
