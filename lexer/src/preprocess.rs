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
    #[test_case("comment/line/1.c", "comment/line/1-ans.c")]
    #[test_case("comment/line/2.c", "comment/line/2-ans.c")]
    #[test_case("comment/line/3.c", "comment/line/3-ans.c")]
    #[test_case("comment/line/4.c", "comment/line/4-ans.c")]
    #[test_case("comment/line/5.c", "comment/line/5-ans.c")]
    #[test_case("comment/line/6.c", "comment/line/6-ans.c")]
    #[test_case("comment/line/7.c", "comment/line/7-ans.c")]
    #[test_case("comment/line/8.c", "comment/line/8-ans.c")]
    #[test_case(
        "comment/line/single-slash/1.txt",
        "comment/line/single-slash/1-ans.txt"
    )]
    #[test_case(
        "comment/line/single-slash/2.txt",
        "comment/line/single-slash/2.txt"
    )]
    #[test_case("comment/block/1.c", "comment/block/1-ans.c")]
    #[test_case("comment/block/2.c", "comment/block/2-ans.c")]
    #[test_case("comment/block/3.c", "comment/block/3-ans.c")]
    #[test_case("comment/block/nested/1.txt", "comment/block/nested/1-ans.txt")]
    #[test_case("comment/mix/1.txt", "comment/mix/1-ans.txt")]
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
