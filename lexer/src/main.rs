mod error;
mod lexer;
mod preprocess;
mod token;

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Only run the preprocessor
    #[arg(short = 'E')]
    preprocessor_only: bool,
    /// Treat the input as if it's already preprocessed
    #[arg(long)]
    preprocessed: bool,

    #[arg(short, long)]
    output: Option<String>,

    file: String,
}

fn preprocessed_path(original_path: &Path) -> PathBuf {
    let mut path = original_path.file_stem().unwrap().to_owned();
    path.push(".i");
    PathBuf::from(path)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file_path = Path::new(&args.file);
    let mut file = File::open(&args.file)
        .with_context(|| format!("cannot open \"{}\"", args.file))?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;
    let preprocessed = if !args.preprocessed {
        match preprocess::preprocess(src.chars()) {
            Ok(s) => {
                if args.preprocessor_only {
                    let output_path = preprocessed_path(file_path);
                    println!("{output_path:?}");
                    let mut output = File::create(output_path)?;
                    output.write_all(s.as_bytes())?;
                    return Ok(());
                }
                s
            }
            Err(line_num) => {
                return Err(error::LexError::PreprocessError {
                    file_path: file_path.to_owned(),
                    source: error::Error {
                        line_num,
                        error_kind: error::ErrorKind::UnterminatedComment,
                    },
                })?
            }
        }
    } else {
        src
    };

    if args.preprocessor_only {
        return Err(anyhow!("expect input to be not have been preprocessed"));
    }

    lexer::scan(&preprocessed)?;

    Ok(())
}
