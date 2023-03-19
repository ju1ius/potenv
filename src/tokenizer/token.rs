use super::pos::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Characters,
    Assign,
    SimpleExpansion,
    StartExpansion,
    ExpansionOperator,
    EndExpansion,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, value: String, position: Position) -> Self {
        Self {
            kind,
            value,
            position,
        }
    }
}
