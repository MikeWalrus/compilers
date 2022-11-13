#[repr(u32)]
pub enum Qualifier {
    Void = 0,
    Int = 1,
    Double = 2,
}

#[repr(u32)]
pub enum Relop {
    Assign = 0,
    Gt = 1,
    Lt = 2,
    Ge = 3,
    Le = 4,
    Neq = 5,
    Eq = 6,
}

#[repr(C, u32)]
pub enum Token {
    IntegerConstant(u32) = 0,
    FloatingConstant(f64) = 1,
    Identifier { offset: usize } = 2,
    If = 3,
    Else = 4,
    While = 5,
    For = 6,
    Do = 7,
    Qualifier(Qualifier) = 8,
    Plus = 9,
    Minus = 10,
    Star = 11,
    Divide = 12,
    Relop(Relop) = 13,
    LeftBrace = 14,
    RightBrace = 15,
    LeftParen = 16,
    RightParen = 17,
    LeftSqBracket = 18,
    RightSqBracket = 19,
    Semicolon = 20,
    Comma = 21,
}
