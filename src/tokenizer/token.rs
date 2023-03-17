use super::pos::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    EOF,
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

impl ToString for Token {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}
