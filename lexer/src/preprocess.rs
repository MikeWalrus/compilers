use crate::token::Position;

pub fn preprocess(
    src: impl IntoIterator<Item = (usize, char)>,
) -> Result<String, Position> {
    let mut i = src.into_iter();
    let mut o = String::new();
    let mut line_num: usize = 1;
    let mut line_start = 0;
    'outer: while let Some(c) = i.next() {
        match c.1 {
            '/' => match i.next() {
                Some(c) => match c.1 {
                    '/' => loop {
                        match i.next() {
                            Some((len, '\n')) => {
                                new_line(
                                    &mut line_num,
                                    &mut line_start,
                                    len,
                                    &mut o,
                                );
                                break;
                            }
                            None => break 'outer,
                            _ => {}
                        }
                    },
                    '*' => {
                        let comment_start = Position {
                            line: line_num,
                            col: c.0 - line_start,
                        };
                        o.push_str("  ");
                        loop {
                            match i.next() {
                                Some((_, '*')) => {
                                    o.push(' ');
                                    match i.next() {
                                        Some((_, '/')) => {
                                            o.push(' ');
                                            break;
                                        }
                                        Some((len, '\n')) => {
                                            new_line(
                                                &mut line_num,
                                                &mut line_start,
                                                len,
                                                &mut o,
                                            );
                                        }
                                        None => return Err(comment_start),
                                        _ => o.push(' '),
                                    }
                                }
                                Some((len, '\n')) => {
                                    new_line(
                                        &mut line_num,
                                        &mut line_start,
                                        len,
                                        &mut o,
                                    );
                                }
                                None => return Err(comment_start),
                                _ => o.push(' '),
                            }
                        }
                    }
                    c => {
                        o.push('/');
                        o.push(c);
                    }
                },
                None => {
                    o.push('/');
                    break;
                }
            },
            _ => {
                if c.1 == '\n' {
                    new_line(&mut line_num, &mut line_start, c.0, &mut o);
                } else {
                    o.push(c.1)
                }
            }
        }
    }
    Ok(o)
}

fn new_line(
    line_num: &mut usize,
    line_start: &mut usize,
    curr_len: usize,
    o: &mut String,
) {
    *line_num += 1;
    *line_start = curr_len + 1;
    o.push('\n')
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    enum Expected {
        AnsFile(&'static str),
        SomeError { pos: Position },
    }
    use Expected::*;

    #[test_case("1.c", AnsFile("1.c"))]
    #[test_case("comment/line/1.c", AnsFile("comment/line/1-ans.c"))]
    #[test_case("comment/line/2.c", AnsFile("comment/line/2-ans.c"))]
    #[test_case("comment/line/3.c", AnsFile("comment/line/3-ans.c"))]
    #[test_case("comment/line/4.c", AnsFile("comment/line/4-ans.c"))]
    #[test_case("comment/line/5.c", AnsFile("comment/line/5-ans.c"))]
    #[test_case("comment/line/6.c", AnsFile("comment/line/6-ans.c"))]
    #[test_case("comment/line/7.c", AnsFile("comment/line/7-ans.c"))]
    #[test_case("comment/line/8.c", AnsFile("comment/line/8-ans.c"))]
    #[test_case(
        "comment/line/single-slash/1.txt",
        AnsFile("comment/line/single-slash/1-ans.txt")
    )]
    #[test_case(
        "comment/line/single-slash/2.txt",
        AnsFile("comment/line/single-slash/2.txt")
    )]
    #[test_case("comment/block/1.c", AnsFile("comment/block/1-ans.c"))]
    #[test_case("comment/block/2.c", AnsFile("comment/block/2-ans.c"))]
    #[test_case("comment/block/3.c", AnsFile("comment/block/3-ans.c"))]
    #[test_case("comment/block/4.c", AnsFile("comment/block/4-ans.c"))]
    #[test_case(
        "comment/block/nested/1.txt",
        AnsFile("comment/block/nested/1-ans.txt")
    )]
    #[test_case("comment/mix/1.txt", AnsFile("comment/mix/1-ans.txt"))]
    #[test_case("comment/mix/2.txt", AnsFile("comment/mix/2-ans.txt"))]
    #[test_case(
        "comment/unterminated/1.txt",
        SomeError{pos: Position{line: 1, col: 1}}
    )]
    #[test_case(
        "comment/unterminated/2.txt",
        SomeError{pos: Position{line: 1, col: 1}}
    )]
    #[test_case(
        "comment/unterminated/3.txt",
        SomeError{pos: Position{line: 4, col: 1}}
    )]
    fn test_preprocess(file: &str, expected: Expected) -> Result<()> {
        let mut file = File::open(String::from("testcase/") + file)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;
        let preprocessed = preprocess(src.char_indices());
        match expected {
            AnsFile(f) => {
                let mut ans_file = File::open(String::from("testcase/") + f)?;
                let mut ans = String::new();
                ans_file.read_to_string(&mut ans)?;
                assert_eq!(preprocessed.unwrap(), ans);
            }
            SomeError { pos } => {
                assert_eq!(preprocessed.unwrap_err(), pos)
            }
        }
        Ok(())
    }
}
