use std::{collections::VecDeque, str::Chars};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    AssignmentList,
    Comment,
    AssignmentName,
    AssignmentValue,
    AssignmentValueEscape,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscape,
    Dollar,
    SimpleExpansion,
    ComplexExpansionStart,
    ComplexExpansion,
    ExpansionOperator,
    ExpansionValue,
    ExpansionValueEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

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

impl Into<String> for Token {
    fn into(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Eof,
    NullCharacter,
    UnescapedSpecialCharacter,
    UnterminatedSingleQuotedString,
    UnterminatedDoubleQuotedString,
    UnsupportedShellParameter,
    UnterminatedExpansion,
    UnsupportedCommandExpansion,
    UnsupportedCommandOrArithmeticExpansion,
    ParseError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
impl std::error::Error for ErrorKind {}

pub type TokenizerResult<T> = Result<T, ErrorKind>;

#[inline(always)]
fn is_wsnl(ch: char) -> bool {
    return matches!(ch, ' ' | '\t' | '\n');
}

#[inline(always)]
fn is_identifier_start(ch: char) -> bool {
    return ch.is_ascii_alphabetic() || ch == '_';
}

#[inline(always)]
fn is_identifier_char(ch: char) -> bool {
    return ch.is_ascii_alphanumeric() || ch == '_';
}

#[inline(always)]
fn is_shell_special_char(ch: char) -> bool {
    return matches!(ch, '`' | '|' | '&' | ';' | '<' | '>' | '(' | ')');
}

#[inline(always)]
fn is_shell_special_param(ch: char) -> bool {
    return ch.is_ascii_digit() || matches!(ch, '@' | '*' | '#' | '?' | '$' | '!' | '-');
}

#[inline(always)]
fn is_dq_escape(ch: char) -> bool {
    return matches!(ch, '"' | '$' | '`' | '\\');
}

#[inline(always)]
fn is_operator(ch: char) -> bool {
    return matches!(ch, '-' | '=' | '+' | '?');
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    input: Chars<'a>,
    done: bool,
    state: State,
    return_states: VecDeque<State>,
    queue: VecDeque<Token>,
    buf: String,
    buf_pos: Position,
    cc: char,
    reconsume: bool,
    line: usize,
    column: usize,
    quoting_stack: VecDeque<Position>,
    expansion_stack: VecDeque<Position>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars(),
            done: false,
            state: State::AssignmentList,
            return_states: VecDeque::with_capacity(16),
            queue: VecDeque::with_capacity(4),
            buf: String::with_capacity(64),
            buf_pos: Position::new(0, 0),
            reconsume: false,
            cc: '\0',
            line: 1,
            column: 0,
            quoting_stack: VecDeque::with_capacity(8),
            expansion_stack: VecDeque::with_capacity(8),
        }
    }

    pub fn next(&mut self) -> TokenizerResult<Token> {
        match self.done {
            false => {
                while self.queue.is_empty() {
                    self.run()?;
                }
                Ok(self.queue.pop_front().unwrap())
            }
            true => {
                if !self.queue.is_empty() {
                    Ok(self.queue.pop_front().unwrap())
                } else {
                    Err(ErrorKind::Eof)
                }
            }
        }
    }

    fn run(&mut self) -> TokenizerResult<()> {
        match self.state {
            State::AssignmentList => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Ok(self.emit_eof()),
                Ok(c) if is_wsnl(c) => Ok(()),
                Ok('#') => Ok(self.switch_to(State::Comment)),
                Ok(c) if is_identifier_start(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::AssignmentName))
                }
                _ => Err(ErrorKind::ParseError),
            },
            State::Comment => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Ok(self.emit_eof()),
                Ok('\n') => Ok(self.switch_to(State::AssignmentList)),
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            },
            State::AssignmentName => match self.consume_the_next_character()? {
                '=' => {
                    self.flush_buffer(TokenKind::Assign);
                    Ok(self.switch_to(State::AssignmentValue))
                }
                c if is_identifier_char(c) => {
                    self.buffer(c);
                    Ok(())
                }
                _ => Err(ErrorKind::ParseError),
            },
            State::AssignmentValue => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => {
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.emit_eof())
                }
                Err(err) => Err(err),
                Ok(c) if is_wsnl(c) => {
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.switch_to(State::AssignmentList))
                }
                Ok('\\') => Ok(self.switch_to(State::AssignmentValueEscape)),
                Ok('\'') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::SingleQuoted))
                }
                Ok('"') => {
                    self.quoting_stack.push_back(self.position());
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Ok('$') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::Dollar))
                }
                Ok(c) if is_shell_special_char(c) => Err(ErrorKind::UnescapedSpecialCharacter),
                Ok(c) => Ok(self.buffer(c)),
            },
            State::AssignmentValueEscape => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => {
                    self.buffer('\\');
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.emit_eof())
                }
                Err(e) => Err(e),
                Ok('\n') => Ok(self.switch_to(State::AssignmentValue)),
                Ok(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::AssignmentValue))
                }
            },
            State::SingleQuoted => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Err(ErrorKind::UnterminatedSingleQuotedString),
                Err(e) => Err(e),
                Ok('\'') => Ok(self.switch_to_return_state()),
                Ok(c) => Ok(self.buffer(c)),
            },
            State::DoubleQuoted => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Err(ErrorKind::UnterminatedDoubleQuotedString),
                Err(e) => Err(e),
                Ok('`') => Err(ErrorKind::UnsupportedCommandExpansion),
                Ok('"') => {
                    self.quoting_stack.pop_back();
                    Ok(self.switch_to_return_state())
                }
                Ok('\\') => Ok(self.switch_to(State::DoubleQuotedEscape)),
                Ok('$') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::Dollar))
                }
                Ok(c) => Ok(self.buffer(c)),
            },
            State::DoubleQuotedEscape => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Err(ErrorKind::UnterminatedDoubleQuotedString),
                Err(e) => Err(e),
                Ok('\n') => Ok(self.switch_to(State::DoubleQuoted)),
                Ok(c) if is_dq_escape(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Ok(c) => {
                    self.buffer('\\');
                    self.buffer(c);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
            },
            State::Dollar => match self.consume_the_next_character() {
                Ok(c) if is_shell_special_param(c) => Err(ErrorKind::UnsupportedShellParameter),
                Ok('(') => Err(ErrorKind::UnsupportedCommandOrArithmeticExpansion),
                Ok('{') => {
                    self.expansion_stack.push_back(self.position());
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.switch_to(State::ComplexExpansionStart))
                }
                Ok(c) if is_identifier_char(c) => {
                    self.flush_buffer(TokenKind::Characters);
                    self.buffer(c);
                    Ok(self.switch_to(State::SimpleExpansion))
                }
                Ok(_) | Err(ErrorKind::Eof) => {
                    self.buffer('$');
                    Ok(self.reconsume_in_return_state())
                }
                Err(e) => Err(e),
            },
            State::SimpleExpansion => match self.consume_the_next_character() {
                Ok(c) if is_identifier_char(c) => Ok(self.buffer(c)),
                _ => {
                    self.flush_buffer(TokenKind::SimpleExpansion);
                    Ok(self.reconsume_in_return_state())
                }
            },
            State::ComplexExpansionStart => match self.consume_the_next_character()? {
                c if is_shell_special_param(c) => Err(ErrorKind::UnsupportedShellParameter),
                c if is_identifier_start(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::ComplexExpansion))
                }
                _ => Err(ErrorKind::ParseError),
            },
            State::ComplexExpansion => match self.consume_the_next_character()? {
                '}' => {
                    self.expansion_stack.pop_back();
                    self.flush_buffer(TokenKind::SimpleExpansion);
                    Ok(self.switch_to_return_state())
                }
                c if is_identifier_char(c) => Ok(self.buffer(c)),
                ':' => {
                    self.flush_buffer(TokenKind::StartExpansion);
                    self.buffer(':');
                    Ok(self.switch_to(State::ExpansionOperator))
                }
                c if is_operator(c) => {
                    self.flush_buffer(TokenKind::StartExpansion);
                    self.emit(TokenKind::ExpansionOperator, c.to_string());
                    Ok(self.switch_to(State::ExpansionValue))
                }
                _ => Err(ErrorKind::ParseError),
            },
            State::ExpansionOperator => match self.consume_the_next_character()? {
                c if is_operator(c) => {
                    self.buffer(c);
                    self.flush_buffer(TokenKind::ExpansionOperator);
                    Ok(self.switch_to(State::ExpansionValue))
                }
                _ => Err(ErrorKind::ParseError),
            },
            State::ExpansionValue => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Err(ErrorKind::UnterminatedExpansion),
                Err(e) => Err(e),
                Ok('`') => Err(ErrorKind::UnsupportedCommandExpansion),
                Ok('}') => {
                    self.expansion_stack.pop_back();
                    self.flush_buffer(TokenKind::Characters);
                    self.emit(TokenKind::EndExpansion, "}".to_string());
                    Ok(self.switch_to_return_state())
                }
                Ok('\\') => Ok(self.switch_to(State::ExpansionValueEscape)),
                Ok('$') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::Dollar))
                }
                Ok('"') => {
                    self.quoting_stack.push_back(self.position());
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Ok('\'') => {
                    if !self.quoting_stack.is_empty() {
                        self.buffer('\'');
                        Ok(())
                    } else {
                        self.return_states.push_back(self.state);
                        Ok(self.switch_to(State::SingleQuoted))
                    }
                }
                Ok(c) => Ok(self.buffer(c)),
            },
            State::ExpansionValueEscape => match self.consume_the_next_character() {
                Err(ErrorKind::Eof) => Err(ErrorKind::UnterminatedExpansion),
                Err(e) => Err(e),
                Ok('\n') => Ok(self.switch_to(State::ExpansionValue)),
                Ok(c) if is_dq_escape(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::ExpansionValue))
                }
                Ok(c) => {
                    if !self.quoting_stack.is_empty() {
                        self.buffer('\\');
                    }
                    self.buffer(c);
                    Ok(self.switch_to(State::ExpansionValue))
                }
            },
        }
    }

    fn switch_to(&mut self, state: State) {
        self.state = state;
    }

    fn switch_to_return_state(&mut self) {
        self.state = self.return_states.pop_back().unwrap();
    }

    fn reconsume_in(&mut self, state: State) {
        self.reconsume = true;
        self.state = state;
    }

    fn reconsume_in_return_state(&mut self) {
        let state = self.return_states.pop_back().unwrap();
        self.reconsume_in(state);
    }

    fn consume_the_next_character(&mut self) -> TokenizerResult<char> {
        if self.reconsume {
            self.reconsume = false;
            Ok(self.cc)
        } else {
            self.input
                .next()
                .ok_or(ErrorKind::Eof)
                .and_then(|c| self.preprocess_char(c))
        }
    }

    fn preprocess_char(&mut self, c: char) -> TokenizerResult<char> {
        if c == '\0' {
            return Err(ErrorKind::NullCharacter);
        }
        if c == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        self.cc = c;
        Ok(c)
    }

    fn emit(&mut self, kind: TokenKind, value: String) {
        self.queue
            .push_back(Token::new(kind, value, self.position()))
    }

    fn emit_eof(&mut self) {
        let pos = Position::new(self.line, self.column + 1);
        self.queue
            .push_back(Token::new(TokenKind::EOF, "".to_string(), pos));
        self.done = true;
    }

    fn flush_buffer(&mut self, kind: TokenKind) {
        if !self.buf.is_empty() {
            self.queue
                .push_back(Token::new(kind, self.buf.clone(), self.buf_pos));
            self.buf.clear();
        }
    }

    fn buffer(&mut self, c: char) {
        if self.buf.is_empty() {
            self.buf_pos = self.position();
        }
        self.buf.push(c);
    }

    fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }
}
