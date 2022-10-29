#[derive(Clone, Copy, Debug)]
pub enum TokenType {
    Comment,
    FunctionDefinition,
    KeywordOther,
    KeywordType,
    Literal,
    Numeric,
    Whitespace,
    Total,
}
