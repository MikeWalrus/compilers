use std::{
    io::{Read, Write},
    mem::{size_of, transmute, MaybeUninit},
    slice,
};

use anyhow::Result;

use crate::{
    lexer::LexerOutput,
    token::{Identifier, TokenKind},
};

#[repr(C)]
struct Section {
    offset: usize,
    len: usize,
}

#[repr(C)]
struct Header {
    token: Section,
    id: Section,
    text: Section,
}

impl Header {
    fn new(lexer_output: &LexerOutput) -> Self {
        let token_offset = size_of::<Header>();
        let id_offset =
            token_offset + lexer_output.tokens.len() * size_of::<TokenKind>();
        let text_offset =
            id_offset + lexer_output.id_table.len() * size_of::<Identifier>();
        Header {
            token: Section {
                offset: token_offset,
                len: lexer_output.tokens.len(),
            },
            id: Section {
                offset: id_offset,
                len: lexer_output.id_table.len(),
            },
            text: Section {
                offset: text_offset,
                len: lexer_output.text.len(),
            },
        }
    }
}

fn write_slice<T>(w: &mut impl Write, t: &[T]) -> Result<()>
where
    T: Sized,
{
    let ptr: *const u8 = t.as_ptr() as *const u8;
    let len = size_of::<T>() * t.len();
    let buf = unsafe { slice::from_raw_parts(ptr, len) };
    w.write_all(buf)?;
    Ok(())
}

pub fn output(w: &mut impl Write, lexer_output: LexerOutput) -> Result<()> {
    let header = Header::new(&lexer_output);
    let header: [u8; size_of::<Header>()] = unsafe { transmute(header) };
    w.write_all(&header)?;
    write_slice(w, &lexer_output.tokens)?;
    write_slice(w, &lexer_output.id_table)?;
    write_slice(w, lexer_output.text.as_bytes())?;
    Ok(())
}

#[allow(clippy::uninit_assumed_init)]
fn load_struct<T>(r: &mut impl Read) -> std::result::Result<T, std::io::Error> {
    let mut s: T = unsafe { MaybeUninit::uninit().assume_init() };
    unsafe {
        let buffer: &mut [u8] = std::slice::from_raw_parts_mut(
            &mut s as *mut T as *mut u8,
            size_of::<T>(),
        );
        r.read_exact(buffer)?;
    };
    Ok(s)
}

fn load_slice<T>(
    r: &mut impl Read,
    len: usize,
) -> std::result::Result<Box<[T]>, std::io::Error> {
    let buf: Box<[MaybeUninit<T>]> = Box::new_uninit_slice(len);
    unsafe {
        let byte_buf: &mut [u8] = std::slice::from_raw_parts_mut(
            buf.as_ptr() as *mut u8,
            len * size_of::<T>(),
        );
        r.read_exact(byte_buf)?;
        Ok(buf.assume_init())
    }
}

fn load_vec<T>(
    r: &mut impl Read,
    len: usize,
) -> std::result::Result<Vec<T>, std::io::Error> {
    load_slice(r, len).map(Vec::from)
}

impl LexerOutput {
    pub fn try_from(mut r: impl Read) -> Result<Self> {
        let header: Header = load_struct(&mut r)?;
        let lexer_output = LexerOutput {
            tokens: load_vec(&mut r, header.token.len)?,
            id_table: load_vec(&mut r, header.id.len)?,
            text: String::from_utf8(load_vec::<u8>(&mut r, header.text.len)?)?,
        };
        Ok(lexer_output)
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;

    use crate::{
        lexer::LexerOutput,
        token::{Identifier, Position, Token, TokenKind},
    };

    use super::output;

    use anyhow::Result;

    #[test]
    fn test_output() -> Result<()> {
        let lexer_output = LexerOutput {
            tokens: vec![
                Token {
                    kind: TokenKind::Id(0),
                    pos: Position { line: 1, col: 1 },
                },
                Token {
                    kind: TokenKind::Else,
                    pos: Position { line: 1, col: 5 },
                },
            ],
            id_table: vec![Identifier {
                text_begin: 0,
                text_len: 3,
            }],
            text: "abc".to_owned(),
        };
        let mut w = BufWriter::new(Vec::new());
        output(&mut w, lexer_output.clone())?;
        let v = w.into_inner()?;
        let lexer_output_read = LexerOutput::try_from(v.as_slice())?;
        assert_eq!(lexer_output, lexer_output_read);
        Ok(())
    }
}
