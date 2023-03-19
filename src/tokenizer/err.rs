use thiserror::Error;

use super::pos::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Eof,
    NullCharacter,
    UnescapedSpecialCharacter(char),
    UnterminatedSingleQuotedString,
    UnterminatedDoubleQuotedString,
    UnsupportedShellParameter(String),
    UnterminatedExpansion,
    UnsupportedCommandExpansion,
    UnsupportedCommandOrArithmeticExpansion,
    InvalidCharacter(char),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => f.write_str("Unexpected end of input"),
            Self::NullCharacter => f.write_str("Unexpected <NUL> character"),
            Self::UnescapedSpecialCharacter(ch) => {
                f.write_fmt(format_args!("Unescaped special shell character '{}'", ch))
            }
            Self::UnterminatedSingleQuotedString => {
                f.write_str("Unterminated single-quoted string")
            }
            Self::UnterminatedDoubleQuotedString => {
                f.write_str("Unterminated double-quoted string")
            }
            Self::UnsupportedShellParameter(p) => {
                f.write_fmt(format_args!("Unsupported special shell parameter: {}", p))
            }
            Self::UnterminatedExpansion => f.write_str("Unterminated expansion"),
            Self::UnsupportedCommandExpansion => f.write_str("Unsupported command expansion"),
            Self::UnsupportedCommandOrArithmeticExpansion => {
                f.write_str("Unsupported command or arithmetic expansion")
            }
            Self::InvalidCharacter(ch) => f.write_fmt(format_args!("Invalid character '{}'", ch)),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub struct SyntaxError {
    kind: ErrorKind,
    position: Position,
    filename: Option<String>,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind().fmt(f)?;
        if let Some(file) = self.file().as_ref() {
            f.write_fmt(format_args!(" in {file}"))?;
        }
        f.write_fmt(format_args!(
            " on line {}, column {}",
            self.line(),
            self.column()
        ))
    }
}

impl SyntaxError {
    pub fn new(kind: ErrorKind, position: Position, filename: Option<String>) -> Self {
        Self {
            kind,
            position,
            filename,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    pub fn line(&self) -> usize {
        self.position.line
    }

    pub fn column(&self) -> usize {
        self.position.column
    }

    pub fn file(&self) -> Option<String> {
        self.filename.clone()
    }
}
