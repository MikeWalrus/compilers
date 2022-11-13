use strum_macros::Display;

struct Error {
    line_num: usize,
    error_type: ErrorKind,
}

#[derive(Debug, Display)]
enum ErrorKind {
    #[strum(serialize = "unterminated comment")]
    UnterminatedComment,
}
