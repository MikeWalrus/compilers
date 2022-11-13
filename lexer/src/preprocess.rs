pub fn preprocess(
    src: impl IntoIterator<Item = char>,
) -> Result<String, usize> {
    let mut i = src.into_iter();
    let mut o = String::new();
    let mut line_num: usize = 1;
    let mut comment_start = line_num;
    'outer: while let Some(c) = i.next() {
        match c {
            '/' => match i.next() {
                Some(c) => match c {
                    '/' => loop {
                        match i.next() {
                            Some('\n') => {
                                line_num += 1;
                                o.push('\n');
                                break;
                            }
                            None => break 'outer,
                            _ => {}
                        }
                    },
                    '*' => {
                        comment_start = line_num;
                        loop {
                            match i.next() {
                                Some('*') => match i.next() {
                                    Some('/') => {
                                        o.push(' ');
                                        break;
                                    }
                                    Some('\n') => {
                                        line_num += 1;
                                        o.push('\n');
                                    }
                                    None => return Err(comment_start),
                                    _ => {}
                                },
                                Some('\n') => {
                                    line_num += 1;
                                    o.push('\n');
                                }
                                None => return Err(comment_start),
                                _ => {}
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
                if c == '\n' {
                    line_num += 1;
                }

                o.push(c)
            }
        }
    }
    Ok(o)
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::*;
    use anyhow::Result;
    use test_case::test_case;

    enum Expected {
        AnsFile(&'static str),
        SomeError { line_num: usize },
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
        SomeError{line_num: 1}
    )]
    #[test_case(
        "comment/unterminated/2.txt",
        SomeError{line_num: 1}
    )]
    #[test_case(
        "comment/unterminated/3.txt",
        SomeError{line_num: 4}
    )]
    fn test_preprocess(file: &str, expected: Expected) -> Result<()> {
        let mut file = File::open(String::from("testcase/") + file)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;
        let preprocessed = preprocess(src.chars());
        match expected {
            AnsFile(f) => {
                let mut ans_file = File::open(String::from("testcase/") + f)?;
                let mut ans = String::new();
                ans_file.read_to_string(&mut ans)?;
                assert_eq!(preprocessed.unwrap(), ans);
            }
            SomeError { line_num } => {
                assert_eq!(preprocessed.unwrap_err(), line_num)
            }
        }
        Ok(())
    }
}
