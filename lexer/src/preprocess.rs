pub fn preprocess(src: impl IntoIterator<Item = char>) -> String {
    let mut i = src.into_iter();
    let mut o = String::new();
    'outer: while let Some(c) = i.next() {
        match c {
            '/' => match i.next() {
                Some(c) => match c {
                    '/' => loop {
                        match i.next() {
                            Some('\n') => {
                                o.push('\n');
                                break;
                            }
                            None => break 'outer,
                            _ => {}
                        }
                    },
                    '*' => loop {
                        match i.next() {
                            Some('*') => match i.next() {
                                Some('/') => {
                                    o.push(' ');
                                    break;
                                }
                                None => todo!("Error"),
                                _ => {}
                            },
                            Some('\n') => {
                                o.push('\n');
                            }
                            None => todo!("Error"),
                            _ => {}
                        }
                    },
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
            _ => o.push(c),
        }
    }
    o
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::*;
    use anyhow::Result;
    use test_case::test_case;

    #[test_case("1.c", "1.c")]
    #[test_case("line-comment/1.c", "line-comment/1-ans.c")]
    #[test_case("line-comment/2.c", "line-comment/2-ans.c")]
    #[test_case("line-comment/3.c", "line-comment/3-ans.c")]
    #[test_case("line-comment/4.c", "line-comment/4-ans.c")]
    #[test_case("line-comment/5.c", "line-comment/5-ans.c")]
    #[test_case("line-comment/6.c", "line-comment/6-ans.c")]
    #[test_case("line-comment/7.c", "line-comment/7-ans.c")]
    #[test_case("line-comment/8.c", "line-comment/8-ans.c")]
    #[test_case(
        "line-comment/single-slash/1.txt",
        "line-comment/single-slash/1-ans.txt"
    )]
    #[test_case(
        "line-comment/single-slash/2.txt",
        "line-comment/single-slash/2.txt"
    )]
    #[test_case("block-comment/1.c", "block-comment/1-ans.c")]
    #[test_case("block-comment/2.c", "block-comment/2-ans.c")]
    #[test_case("block-comment/3.c", "block-comment/3-ans.c")]
    #[test_case("block-comment/nested/1.txt", "block-comment/nested/1-ans.txt")]
    #[test_case("mix-comment/1.txt", "mix-comment/1-ans.txt")]
    fn test_preprocess(file: &str, ans_file: &str) -> Result<()> {
        let mut file = File::open(String::from("testcase/") + file)?;
        let mut ans_file = File::open(String::from("testcase/") + ans_file)?;
        let mut src = String::new();
        let mut ans = String::new();
        file.read_to_string(&mut src)?;
        let preprocessed = preprocess(src.chars());
        ans_file.read_to_string(&mut ans)?;
        assert_eq!(preprocessed, ans);
        Ok(())
    }
}
