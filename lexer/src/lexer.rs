use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::iter::Peekable;

use crate::util::ignore_num_ref;

use super::error::*;
use super::token::*;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LexerOutput {
    pub tokens: Vec<Token>,
    pub id_table: Vec<Identifier>,
    pub text: String,
}

pub fn scan(s: &str) -> Result<LexerOutput, Error> {
    let lines = s.split('\n');
    let mut text = String::with_capacity(1024);
    let mut known_ids = HashMap::new();
    let mut tokens = Vec::new();
    let keyword_map = keyword_map();
    let mut id_table = Vec::new();
    for (line_num, line) in lines.enumerate() {
        let mut i = line.char_indices().peekable();
        loop {
            if i.peek().is_none() {
                break;
            }
            if let Some(token) = get_token(
                &mut i,
                &mut text,
                &keyword_map,
                &mut known_ids,
                &mut id_table,
                line_num,
            )? {
                tokens.push(token);
            }
        }
    }
    Ok(LexerOutput {
        tokens,
        id_table,
        text,
    })
}

fn get_token_after_decimal_point(
    i: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    last_match_int: Option<u32>,
) -> Option<TokenKind> {
    let integer = if let Some(n) = last_match_int { n } else { 0 };
    let mut floating_constant: f64 = 0.;
    let mut n = 0.1;
    match ignore_num_ref(i.peek()) {
        Some(c @ '0'..='9') => {
            floating_constant += n * c.to_digit(10).unwrap() as f64
        }
        _ => {
            return if last_match_int.is_none() {
                None
            } else {
                Some(TokenKind::FloatingConstant(integer as f64))
            }
        }
    }

    loop {
        n *= 0.1;
        i.next().unwrap();
        match ignore_num_ref(i.peek()) {
            Some(c @ '0'..='9') => {
                floating_constant += n * c.to_digit(10).unwrap() as f64
            }
            _ => break,
        }
    }

    Some(TokenKind::FloatingConstant(
        integer as f64 + floating_constant,
    ))
}

fn get_token_int(
    i: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    last_match_int: u32,
) -> u32 {
    let mut n = last_match_int;
    loop {
        match i.peek() {
            Some((_, c @ '0'..='9')) => n = n * 10 + c.to_digit(10).unwrap(),
            _ => return n,
        }
        i.next().unwrap();
    }
}

fn get_token_identifier(
    i: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    first_char: char,
    text: &mut String,
    keyword_map: &HashMap<&str, TokenKind>,
    id_map: &mut HashMap<String, usize>,
    id_table: &mut Vec<Identifier>,
) -> TokenKind {
    let mut new_id = String::from(first_char);
    while let Some(&(_, c)) = i.peek() {
        if c.is_whitespace() || (c != '_' && c.is_ascii_punctuation()) {
            break;
        }
        i.next().unwrap();
        new_id.push(c);
    }
    if let Some(token) = keyword_map.get(new_id.as_str()) {
        token.clone()
    } else {
        match id_map.entry(new_id) {
            Entry::Occupied(e) => TokenKind::Id(*e.get()),
            Entry::Vacant(e) => {
                let text_begin = text.len();
                let id = e.key();
                let text_len = id.len();
                text.push_str(id);
                let id_index = id_table.len();
                e.insert(id_index);
                id_table.push(Identifier {
                    text_begin,
                    text_len,
                });
                TokenKind::Id(id_index)
            }
        }
    }
}

fn get_token(
    i: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    text: &mut String,
    keyword_map: &HashMap<&str, TokenKind>,
    known_ids: &mut HashMap<String, usize>,
    id_table: &mut Vec<Identifier>,
    line_num: usize,
) -> Result<Option<Token>, Error> {
    let _text_offset = text.len();
    let (token_start_col, c) = i.next().unwrap();

    Ok(match c {
        '.' => {
            let token = get_token_after_decimal_point(i, None);
            match token {
                Some(t) => Some(t),
                None => {
                    return Err(Error {
                        pos: Position {
                            line: line_num + 1,
                            col: token_start_col + 1,
                        },
                        error_kind: ErrorKind::ExpectDigit,
                    })
                }
            }
        }
        '0' => {
            if let Some('.') = ignore_num_ref(i.peek()) {
                i.next().unwrap();
                get_token_after_decimal_point(i, Some(0))
            } else {
                Some(TokenKind::IntegerConstant(0))
            }
        }
        c @ '1'..='9' => {
            let int = Some(get_token_int(i, c.to_digit(10).unwrap()));
            if let Some('.') = ignore_num_ref(i.peek()) {
                i.next().unwrap();
                get_token_after_decimal_point(i, int)
            } else {
                int.map(TokenKind::IntegerConstant)
            }
        }
        '+' => Some(TokenKind::Plus),
        '-' => Some(TokenKind::Minus),
        '*' => Some(TokenKind::Star),
        '/' => Some(TokenKind::Divide),
        '(' => Some(TokenKind::LeftParen),
        ')' => Some(TokenKind::RightParen),
        '[' => Some(TokenKind::LeftSqBracket),
        ']' => Some(TokenKind::RightSqBracket),
        '{' => Some(TokenKind::LeftBrace),
        '}' => Some(TokenKind::RightBrace),
        ';' => Some(TokenKind::Semicolon),
        ',' => Some(TokenKind::Comma),
        '=' => Some(match ignore_num_ref(i.peek()) {
            Some('=') => {
                i.next().unwrap();
                TokenKind::Relop(RelopKind::Eq)
            }
            _ => TokenKind::Relop(RelopKind::Assign),
        }),
        '>' => Some(match ignore_num_ref(i.peek()) {
            Some('=') => {
                i.next().unwrap();
                TokenKind::Relop(RelopKind::Ge)
            }
            _ => TokenKind::Relop(RelopKind::Gt),
        }),
        '<' => Some(match ignore_num_ref(i.peek()) {
            Some('=') => {
                i.next().unwrap();
                TokenKind::Relop(RelopKind::Le)
            }
            _ => TokenKind::Relop(RelopKind::Lt),
        }),
        '!' => Some(match ignore_num_ref(i.peek()) {
            Some('=') => {
                i.next().unwrap();
                TokenKind::Relop(RelopKind::Neq)
            }
            _ => TokenKind::Not,
        }),
        _ => {
            if c.is_whitespace() {
                None
            } else {
                Some(get_token_identifier(
                    i,
                    c,
                    text,
                    keyword_map,
                    known_ids,
                    id_table,
                ))
            }
        }
    }
    .map(|kind| Token {
        kind,
        pos: Position {
            line: line_num + 1,
            col: token_start_col + 1,
        },
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use test_case::test_case;
    use QualifierKind::*;
    use RelopKind::*;
    use TokenKind::*;

    #[test_case("123", Ok(vec![(IntegerConstant(123), 1, 1)]))]
    #[test_case("\n123", Ok(vec![(IntegerConstant(123), 2, 1)]))]
    #[test_case("0", Ok(vec![(IntegerConstant(0), 1, 1)]))]
    #[test_case("0.1", Ok(vec![(FloatingConstant(0.1), 1, 1)]))]
    #[test_case("1.", Ok(vec![(FloatingConstant(1.0), 1, 1)]))]
    #[test_case("0. \n", Ok(vec![(FloatingConstant(0.0), 1, 1)]))]
    #[test_case(".0", Ok(vec![(FloatingConstant(0.0), 1, 1)]))]
    #[test_case("1.2", Ok(vec![(FloatingConstant(1.2), 1, 1)]))]
    #[test_case("1.2345", Ok(vec![(FloatingConstant(1.2345), 1, 1)]))]
    #[test_case("1.2345 1", Ok(vec![(FloatingConstant(1.2345), 1, 1), (IntegerConstant(1), 1, 8)]))]
    #[test_case("0.1 1.2", Ok(vec![(FloatingConstant(0.1), 1, 1), (FloatingConstant(1.2), 1, 5)]))]
    #[test_case("1 2 3 1.2 2.3", Ok(vec![
        (IntegerConstant(1), 1, 1),
        (IntegerConstant(2), 1, 3),
        (IntegerConstant(3), 1, 5),
        (FloatingConstant(1.2), 1, 7),
        (FloatingConstant(2.3), 1, 11)
    ]))]
    #[test_case("()[]{}", Ok(vec![
        (LeftParen, 1, 1),
        (RightParen, 1, 2),
        (LeftSqBracket, 1, 3),
        (RightSqBracket, 1, 4),
        (LeftBrace, 1, 5),
        (RightBrace, 1, 6)
    ]))]
    #[test_case("+-*/", Ok(vec![
        (Plus, 1, 1),
        (Minus, 1, 2),
        (Star, 1, 3),
        (Divide, 1, 4)
    ]))]
    #[test_case(";,", Ok(vec![(Semicolon, 1, 1), (Comma, 1, 2)]))]
    #[test_case("1+1/2", Ok(vec![
        (IntegerConstant(1), 1, 1),
        (Plus, 1, 2),
        (IntegerConstant(1), 1, 3),
        (Divide, 1, 4),
        (IntegerConstant(2), 1, 5)
    ]))]
    #[test_case("1.2/2", Ok(vec![
        (FloatingConstant(1.2), 1, 1),
        (Divide, 1, 4),
        (IntegerConstant(2), 1, 5)
    ]))]
    #[test_case("0.+.0", Ok(vec![
        (FloatingConstant(0.0), 1, 1),
        (Plus, 1, 3),
        (FloatingConstant(0.0), 1, 4)
    ]))]
    #[test_case("=", Ok(vec![(Relop(RelopKind::Assign), 1, 1)]))]
    #[test_case("==", Ok(vec![(Relop(RelopKind::Eq), 1, 1)]))]
    #[test_case("<", Ok(vec![(Relop(RelopKind::Lt), 1, 1)]))]
    #[test_case("<=", Ok(vec![(Relop(RelopKind::Le), 1, 1)]))]
    #[test_case(">", Ok(vec![(Relop(RelopKind::Gt), 1, 1)]))]
    #[test_case(">=", Ok(vec![(Relop(RelopKind::Ge), 1, 1)]))]
    #[test_case("!", Ok(vec![(Not, 1, 1)]))]
    #[test_case("!=", Ok(vec![(Relop(RelopKind::Neq), 1, 1)]))]
    #[test_case("!==<>==<=>=", Ok(vec![
        (Relop(RelopKind::Neq), 1, 1),
        (Relop(RelopKind::Assign), 1, 3),
        (Relop(RelopKind::Lt), 1, 4),
        (Relop(RelopKind::Ge), 1, 5),
        (Relop(RelopKind::Assign), 1, 7),
        (Relop(RelopKind::Le), 1, 8),
        (Relop(RelopKind::Ge), 1, 10)
    ]))]
    #[test_case("1>=2=2", Ok(vec![
        (IntegerConstant(1), 1, 1),
        (Relop(RelopKind::Ge), 1, 2),
        (IntegerConstant(2), 1, 4),
        (Relop(RelopKind::Assign), 1, 5),
        (IntegerConstant(2), 1, 6)
    ]))]
    #[test_case(".", Err(Error{pos: Position{line: 1, col: 1}, error_kind: ErrorKind::ExpectDigit}))]
    #[test_case(". 1", Err(Error{pos: Position{line: 1, col: 1}, error_kind: ErrorKind::ExpectDigit}))]
    fn test_scan_without_text(
        s: &str,
        ans: Result<Vec<(TokenKind, usize, usize)>, Error>,
    ) -> Result<()> {
        assert_eq!(
            scan(s).map(|x| x.tokens),
            ans.map(|v| {
                v.into_iter()
                    .map(|(kind, line, col)| Token {
                        kind,
                        pos: Position { line, col },
                    })
                    .collect()
            })
        );
        Ok(())
    }

    struct TokenTestcase {
        s: &'static str,
        ans: LexerOutput,
    }

    macro_rules! test_dir {
        () => {
            "../testcase/token/"
        };
    }

    macro_rules! include_test_str {
        ($file:expr) => {
            include_str!(concat!(test_dir!(), $file))
        };
    }

    macro_rules! include_test {
        ($file:expr) => {
            include!(concat!(test_dir!(), $file))
        };
    }

    macro_rules! token_testcase {
        ($name:literal) => {
            TokenTestcase {
                s: include_test_str!(concat!($name, ".in")),
                ans: LexerOutput {
                    tokens: include_test!(concat!($name, ".token.in")),
                    id_table: include_test!(concat!($name, ".id.in")),
                    text: include_test_str!(concat!($name, ".text.in"))
                        .to_owned(),
                },
            }
        };
    }

    #[test_case(token_testcase!{1})]
    #[test_case(token_testcase!{2})]
    #[test_case(token_testcase!{3})]
    #[test_case(token_testcase!{4})]
    #[test_case(token_testcase!{5})]
    #[test_case(token_testcase!{6})]
    fn test_scan(t: TokenTestcase) -> Result<()> {
        let result = scan(t.s)?;
        assert_eq!(result, t.ans);
        Ok(())
    }
}
