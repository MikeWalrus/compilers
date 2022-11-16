use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::iter::Peekable;

use super::error::*;
use super::token::*;

#[derive(Debug)]
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
        let mut i = line.chars().peekable();
        loop {
            match get_token(
                &mut i,
                &mut text,
                &keyword_map,
                &mut known_ids,
                &mut id_table,
            ) {
                Some(Some(Ok(token))) => {
                    tokens.push(token);
                }
                Some(Some(Err(e))) => {
                    return Err(Error {
                        line_num: line_num + 1,
                        error_kind: e,
                    })
                }
                None => break,
                _ => {}
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
    i: &mut Peekable<impl Iterator<Item = char>>,
    last_match_int: Option<u32>,
) -> Option<Token> {
    let integer = if let Some(n) = last_match_int { n } else { 0 };
    let mut floating_constant: f64 = 0.;
    let mut n = 0.1;
    match i.peek() {
        Some(c @ '0'..='9') => {
            floating_constant += n * c.to_digit(10).unwrap() as f64
        }
        _ => {
            return if last_match_int.is_none() {
                None
            } else {
                Some(Token::FloatingConstant(integer as f64))
            }
        }
    }

    loop {
        n *= 0.1;
        i.next().unwrap();
        match i.peek() {
            Some(c @ '0'..='9') => {
                floating_constant += n * c.to_digit(10).unwrap() as f64
            }
            _ => break,
        }
    }

    Some(Token::FloatingConstant(integer as f64 + floating_constant))
}

fn get_token_int(
    i: &mut Peekable<impl Iterator<Item = char>>,
    last_match_int: u32,
) -> u32 {
    let mut n = last_match_int;
    loop {
        match i.peek() {
            Some(c @ '0'..='9') => n = n * 10 + c.to_digit(10).unwrap(),
            _ => return n,
        }
        i.next().unwrap();
    }
}

fn get_token_identifier(
    i: &mut Peekable<impl Iterator<Item = char>>,
    first_char: char,
    text: &mut String,
    keyword_map: &HashMap<&str, Token>,
    id_map: &mut HashMap<String, usize>,
    id_table: &mut Vec<Identifier>,
) -> Token {
    let mut new_id = String::from(first_char);
    while let Some(&c) = i.peek() {
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
            Entry::Occupied(e) => Token::Id(*e.get()),
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
                Token::Id(id_index)
            }
        }
    }
}

fn get_token(
    i: &mut Peekable<impl Iterator<Item = char>>,
    text: &mut String,
    keyword_map: &HashMap<&str, Token>,
    known_ids: &mut HashMap<String, usize>,
    id_table: &mut Vec<Identifier>,
) -> Option<Option<Result<Token, ErrorKind>>> {
    let _text_offset = text.len();
    let c = i.next()?;

    Some(
        match c {
            '.' => {
                let token = get_token_after_decimal_point(i, None);
                match token {
                    Some(t) => Some(t),
                    None => return Some(Some(Err(ErrorKind::ExpectDigit))),
                }
            }
            '0' => {
                if let Some('.') = i.peek() {
                    i.next().unwrap();
                    get_token_after_decimal_point(i, Some(0))
                } else {
                    Some(Token::IntegerConstant(0))
                }
            }
            c @ '1'..='9' => {
                let int = Some(get_token_int(i, c.to_digit(10).unwrap()));
                if let Some('.') = i.peek() {
                    i.next().unwrap();
                    get_token_after_decimal_point(i, int)
                } else {
                    int.map(Token::IntegerConstant)
                }
            }
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Divide),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '[' => Some(Token::LeftSqBracket),
            ']' => Some(Token::RightSqBracket),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),
            '=' => Some(match i.peek() {
                Some('=') => {
                    i.next().unwrap();
                    Token::Relop(RelopKind::Eq)
                }
                _ => Token::Relop(RelopKind::Assign),
            }),
            '>' => Some(match i.peek() {
                Some('=') => {
                    i.next().unwrap();
                    Token::Relop(RelopKind::Ge)
                }
                _ => Token::Relop(RelopKind::Gt),
            }),
            '<' => Some(match i.peek() {
                Some('=') => {
                    i.next().unwrap();
                    Token::Relop(RelopKind::Le)
                }
                _ => Token::Relop(RelopKind::Lt),
            }),
            '!' => Some(match i.peek() {
                Some('=') => {
                    i.next().unwrap();
                    Token::Relop(RelopKind::Neq)
                }
                _ => Token::Not,
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
        .map(Ok),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use test_case::test_case;
    use QualifierKind::*;
    use Token::*;

    #[test_case("123", Ok(vec![IntegerConstant(123)]))]
    #[test_case("123 \n", Ok(vec![IntegerConstant(123)]))]
    #[test_case("0", Ok(vec![IntegerConstant(0)]))]
    #[test_case("0.1", Ok(vec![FloatingConstant(0.1)]))]
    #[test_case("1.", Ok(vec![FloatingConstant(1.0)]))]
    #[test_case("0. \n", Ok(vec![FloatingConstant(0.0)]))]
    #[test_case(".0", Ok(vec![FloatingConstant(0.0)]))]
    #[test_case("1.2", Ok(vec![FloatingConstant(1.2)]))]
    #[test_case("1.2345", Ok(vec![FloatingConstant(1.2345)]))]
    #[test_case("1.2345 1", Ok(vec![FloatingConstant(1.2345), IntegerConstant(1)]))]
    #[test_case("0.1 1.2", Ok(vec![FloatingConstant(0.1), FloatingConstant(1.2)]))]
    #[test_case("1 2 3 1.2 2.3", Ok(vec![IntegerConstant(1), IntegerConstant(2), IntegerConstant(3), FloatingConstant(1.2), FloatingConstant(2.3)]))]
    #[test_case("()[]{}", Ok(vec![LeftParen, RightParen, LeftSqBracket, RightSqBracket, LeftBrace, RightBrace]))]
    #[test_case("+-*/", Ok(vec![Plus, Minus, Star, Divide]))]
    #[test_case(";,", Ok(vec![Semicolon, Comma]))]
    #[test_case("1+1/2", Ok(vec![IntegerConstant(1), Plus, IntegerConstant(1), Divide, IntegerConstant(2)]))]
    #[test_case("1.2/2", Ok(vec![FloatingConstant(1.2), Divide, IntegerConstant(2)]))]
    #[test_case("0.+.0", Ok(vec![FloatingConstant(0.0), Plus, FloatingConstant(0.0)]))]
    #[test_case("=", Ok(vec![Relop(RelopKind::Assign)]))]
    #[test_case("==", Ok(vec![Relop(RelopKind::Eq)]))]
    #[test_case("<", Ok(vec![Relop(RelopKind::Lt)]))]
    #[test_case("<=", Ok(vec![Relop(RelopKind::Le)]))]
    #[test_case(">", Ok(vec![Relop(RelopKind::Gt)]))]
    #[test_case(">=", Ok(vec![Relop(RelopKind::Ge)]))]
    #[test_case("!", Ok(vec![Not]))]
    #[test_case("!=", Ok(vec![Relop(RelopKind::Neq)]))]
    #[test_case("!==<>==<=>=", Ok(vec![Relop(RelopKind::Neq), Relop(RelopKind::Assign), Relop(RelopKind::Lt), Relop(RelopKind::Ge), Relop(RelopKind::Assign), Relop(RelopKind::Le), Relop(RelopKind::Ge)]))]
    #[test_case("1>=2=2", Ok(vec![IntegerConstant(1), Relop(RelopKind::Ge), IntegerConstant(2), Relop(RelopKind::Assign), IntegerConstant(2)]))]
    #[test_case(".", Err(Error{line_num: 1, error_kind: ErrorKind::ExpectDigit}))]
    #[test_case(". 1", Err(Error{line_num: 1, error_kind: ErrorKind::ExpectDigit}))]
    fn test_scan_without_text(
        s: &str,
        ans: Result<Vec<Token>, Error>,
    ) -> Result<()> {
        assert_eq!(scan(s).map(|x| x.tokens), ans);
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
