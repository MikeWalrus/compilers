#![feature(new_uninit)]
#![feature(never_type)]
mod error;
mod lexer;
mod persist;
mod preprocess;
mod token;
mod util;

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use lexer::LexerOutput;
use persist::output;

#[derive(Parser)]
struct Args {
    /// Only run the preprocessor
    #[arg(short = 'E')]
    preprocessor_only: bool,
    /// Treat the input as if it's already preprocessed
    #[arg(long)]
    preprocessed: bool,
    /// Output human-readable tokens as well
    #[arg(short('H'), long)]
    human_readable: bool,
    /// Read and show lexer output only
    #[arg(short, long)]
    show_output: bool,

    #[arg(short, long)]
    output: Option<String>,

    file: String,
}

impl Args {
    fn output_file<F>(self, rename: F) -> Result<File, std::io::Error>
    where
        F: Fn(&str) -> String,
    {
        File::create(self.output.unwrap_or_else(|| rename(&self.file)))
    }
}

fn modify_ext(original_path: &str, ext: &str) -> String {
    let mut path = PathBuf::from(original_path).file_stem().unwrap().to_owned();
    path.push(ext);
    path.into_string().unwrap()
}

fn preprocessed_path(original_path: &str) -> String {
    modify_ext(original_path, ".i")
}

fn lexer_output_path(original_path: &str) -> String {
    modify_ext(original_path, ".lexeroutput")
}
fn main() -> Result<()> {
    let args = Args::parse();
    let file_path = Path::new(&args.file);
    let mut file = File::open(&args.file)
        .with_context(|| format!("cannot open \"{}\"", args.file))?;
    if args.show_output {
        let lexer_output = LexerOutput::try_from(file)
            .context("corrupted lexer output file")?;
        println!("{:#?}", lexer_output);
        return Ok(());
    }
    let mut src = String::new();
    file.read_to_string(&mut src)?;
    let preprocessed = if !args.preprocessed {
        preprocess::preprocess(src.char_indices()).map_err(|e| {
            error::Error {
                pos: e,
                error_kind: error::ErrorKind::UnterminatedComment,
            }
            .report(file_path)
            .unwrap_err()
        })?
    } else {
        src
    };

    if args.preprocessor_only {
        let mut output = args
            .output_file(preprocessed_path)
            .context("cannot create file for preprocessor output")?;
        output.write_all(preprocessed.as_bytes())?;
        return Ok(());
    }

    if args.preprocessor_only {
        return Err(anyhow!("expect input to not have been preprocessed"));
    }

    let lexer_output = lexer::scan(&preprocessed)
        .unwrap_or_else(|e| e.report(file_path).unwrap());

    if args.human_readable {
        eprintln!("{:#?}", lexer_output);
    }

    let mut output_file = args
        .output_file(lexer_output_path)
        .context("cannot create file for lexer output")?;

    output(&mut output_file, lexer_output)?;

    Ok(())
}
