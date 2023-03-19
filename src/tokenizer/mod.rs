use std::collections::VecDeque;

use self::{
    err::{ErrorKind, SyntaxError},
    pos::Position,
    token::*,
};

pub mod err;
pub mod pos;
#[cfg(test)]
mod tests;
pub mod token;

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

pub type TokenizerResult = Result<Token, SyntaxError>;

#[inline(always)]
fn is_wsnl(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n')
}

#[inline(always)]
fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

#[inline(always)]
fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[inline(always)]
fn is_shell_special_char(ch: char) -> bool {
    matches!(ch, '|' | '&' | ';' | '<' | '>' | '(' | ')')
}

#[inline(always)]
fn is_shell_special_param(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '@' | '*' | '#' | '?' | '$' | '!' | '-')
}

#[inline(always)]
fn is_dq_escape(ch: char) -> bool {
    matches!(ch, '"' | '$' | '`' | '\\')
}

#[inline(always)]
fn is_operator(ch: char) -> bool {
    matches!(ch, '-' | '=' | '+' | '?')
}

#[derive(Debug)]
pub struct Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    input: I,
    filename: Option<String>,
    done: bool,
    state: State,
    return_states: VecDeque<State>,
    queue: VecDeque<Token>,
    buf: String,
    buf_pos: Position,
    cc: Option<char>,
    reconsume: bool,
    line: usize,
    column: usize,
    single_quote_pos: Position,
    quoting_stack: VecDeque<Position>,
    expansion_stack: VecDeque<Position>,
}

impl<I> Iterator for Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    type Item = TokenizerResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            if !self.queue.is_empty() {
                Some(Ok(self.queue.pop_front().unwrap()))
            } else {
                None
            }
        } else {
            while self.queue.is_empty() {
                if let Err(e) = self.run() {
                    return Some(Err(e));
                }
            }
            Some(Ok(self.queue.pop_front().unwrap()))
        }
    }
}

impl<I> Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(input: I, filename: Option<String>) -> Self {
        Self {
            input,
            filename,
            done: false,
            state: State::AssignmentList,
            return_states: VecDeque::with_capacity(16),
            queue: VecDeque::with_capacity(4),
            buf: String::with_capacity(64),
            buf_pos: Position::new(0, 0),
            reconsume: false,
            cc: None,
            line: 1,
            column: 0,
            single_quote_pos: Position::new(0, 0),
            quoting_stack: VecDeque::with_capacity(8),
            expansion_stack: VecDeque::with_capacity(8),
        }
    }

    #[allow(clippy::unit_arg)]
    fn run(&mut self) -> Result<(), SyntaxError> {
        match self.state {
            State::AssignmentList => match self.consume_the_next_character() {
                None => Ok(self.emit_eof()),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_wsnl(c) => Ok(()),
                Some('#') => Ok(self.switch_to(State::Comment)),
                Some(c) if is_identifier_start(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::AssignmentName))
                }
                Some(c) => self.err(ErrorKind::InvalidCharacter(c)),
            },
            State::Comment => loop {
                match self.consume_the_next_character() {
                    None => return Ok(self.emit_eof()),
                    Some('\0') => return self.err(ErrorKind::NullCharacter),
                    Some('\n') => return Ok(self.switch_to(State::AssignmentList)),
                    Some(_) => (),
                };
            },
            State::AssignmentName => match self.consume_the_next_character() {
                None => self.err_eof(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('=') => {
                    self.flush_buffer(TokenKind::Assign);
                    Ok(self.switch_to(State::AssignmentValue))
                }
                Some(c) if is_identifier_char(c) => {
                    self.buffer(c);
                    Ok(())
                }
                Some(c) => self.err(ErrorKind::InvalidCharacter(c)),
            },
            State::AssignmentValue => match self.consume_the_next_character() {
                None => {
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.emit_eof())
                }
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_wsnl(c) => {
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.switch_to(State::AssignmentList))
                }
                Some('\\') => Ok(self.switch_to(State::AssignmentValueEscape)),
                Some('\'') => {
                    self.single_quote_pos = self.cur_pos();
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::SingleQuoted))
                }
                Some('"') => {
                    self.quoting_stack.push_back(self.cur_pos());
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Some('$') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::Dollar))
                }
                Some('`') => self.err(ErrorKind::UnsupportedCommandExpansion),
                Some(c) if is_shell_special_char(c) => {
                    self.err(ErrorKind::UnescapedSpecialCharacter(c))
                }
                Some(c) => Ok(self.buffer(c)),
            },
            State::AssignmentValueEscape => match self.consume_the_next_character() {
                None => {
                    self.buffer('\\');
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.emit_eof())
                }
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('\n') => Ok(self.switch_to(State::AssignmentValue)),
                Some(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::AssignmentValue))
                }
            },
            State::SingleQuoted => loop {
                match self.consume_the_next_character() {
                    None => return self.unterminated_single_quote(),
                    Some('\0') => return self.err(ErrorKind::NullCharacter),
                    Some('\'') => return Ok(self.switch_to_return_state()),
                    Some(c) => self.buffer(c),
                };
            },
            State::DoubleQuoted => loop {
                match self.consume_the_next_character() {
                    None => return self.unterminated_double_quote(),
                    Some('\0') => return self.err(ErrorKind::NullCharacter),
                    Some('`') => return self.err(ErrorKind::UnsupportedCommandExpansion),
                    Some('"') => {
                        self.quoting_stack.pop_back();
                        return Ok(self.switch_to_return_state());
                    }
                    Some('\\') => return Ok(self.switch_to(State::DoubleQuotedEscape)),
                    Some('$') => {
                        self.return_states.push_back(self.state);
                        return Ok(self.switch_to(State::Dollar));
                    }
                    Some(c) => self.buffer(c),
                };
            },
            State::DoubleQuotedEscape => match self.consume_the_next_character() {
                None => self.unterminated_double_quote(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('\n') => Ok(self.switch_to(State::DoubleQuoted)),
                Some(c) if is_dq_escape(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Some(c) => {
                    self.buffer('\\');
                    self.buffer(c);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
            },
            State::Dollar => match self.consume_the_next_character() {
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_shell_special_param(c) => {
                    self.err(ErrorKind::UnsupportedShellParameter(format!("${}", c)))
                }
                Some('(') => self.err(ErrorKind::UnsupportedCommandOrArithmeticExpansion),
                Some('{') => {
                    self.expansion_stack.push_back(self.cur_pos());
                    self.flush_buffer(TokenKind::Characters);
                    Ok(self.switch_to(State::ComplexExpansionStart))
                }
                Some(c) if is_identifier_char(c) => {
                    self.flush_buffer(TokenKind::Characters);
                    self.buffer(c);
                    Ok(self.switch_to(State::SimpleExpansion))
                }
                Some(_) | None => {
                    self.buffer('$');
                    Ok(self.reconsume_in_return_state())
                }
            },
            State::SimpleExpansion => match self.consume_the_next_character() {
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_identifier_char(c) => Ok(self.buffer(c)),
                _ => {
                    self.flush_buffer(TokenKind::SimpleExpansion);
                    Ok(self.reconsume_in_return_state())
                }
            },
            State::ComplexExpansionStart => match self.consume_the_next_character() {
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_shell_special_param(c) => {
                    self.err(ErrorKind::UnsupportedShellParameter(format!("${{{}}}", c)))
                }
                Some(c) if is_identifier_start(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::ComplexExpansion))
                }
                Some(c) => self.err(ErrorKind::InvalidCharacter(c)),
                None => self.err_eof(),
            },
            State::ComplexExpansion => match self.consume_the_next_character() {
                None => self.unterminated_expansion(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('}') => {
                    self.expansion_stack.pop_back();
                    self.flush_buffer(TokenKind::SimpleExpansion);
                    Ok(self.switch_to_return_state())
                }
                Some(c) if is_identifier_char(c) => Ok(self.buffer(c)),
                Some(':') => {
                    self.flush_buffer(TokenKind::StartExpansion);
                    self.buffer(':');
                    Ok(self.switch_to(State::ExpansionOperator))
                }
                Some(c) if is_operator(c) => {
                    self.flush_buffer(TokenKind::StartExpansion);
                    self.emit(TokenKind::ExpansionOperator, c.to_string());
                    Ok(self.switch_to(State::ExpansionValue))
                }
                Some(c) => self.err(ErrorKind::InvalidCharacter(c)),
            },
            State::ExpansionOperator => match self.consume_the_next_character() {
                None => self.err_eof(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some(c) if is_operator(c) => {
                    self.buffer(c);
                    self.flush_buffer(TokenKind::ExpansionOperator);
                    Ok(self.switch_to(State::ExpansionValue))
                }
                Some(c) => self.err(ErrorKind::InvalidCharacter(c)),
            },
            State::ExpansionValue => match self.consume_the_next_character() {
                None => self.unterminated_expansion(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('`') => self.err(ErrorKind::UnsupportedCommandExpansion),
                Some('}') => {
                    self.expansion_stack.pop_back();
                    self.flush_buffer(TokenKind::Characters);
                    self.emit(TokenKind::EndExpansion, "}".to_string());
                    Ok(self.switch_to_return_state())
                }
                Some('\\') => Ok(self.switch_to(State::ExpansionValueEscape)),
                Some('$') => {
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::Dollar))
                }
                Some('"') => {
                    self.quoting_stack.push_back(self.cur_pos());
                    self.return_states.push_back(self.state);
                    Ok(self.switch_to(State::DoubleQuoted))
                }
                Some('\'') => {
                    if !self.quoting_stack.is_empty() {
                        self.buffer('\'');
                        Ok(())
                    } else {
                        self.single_quote_pos = self.cur_pos();
                        self.return_states.push_back(self.state);
                        Ok(self.switch_to(State::SingleQuoted))
                    }
                }
                Some(c) => Ok(self.buffer(c)),
            },
            State::ExpansionValueEscape => match self.consume_the_next_character() {
                None => self.unterminated_expansion(),
                Some('\0') => self.err(ErrorKind::NullCharacter),
                Some('\n') => Ok(self.switch_to(State::ExpansionValue)),
                Some(c) if is_dq_escape(c) => {
                    self.buffer(c);
                    Ok(self.switch_to(State::ExpansionValue))
                }
                Some(c) => {
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

    fn consume_the_next_character(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
        } else {
            self.cc = self.input.next().map(|c| {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
                c
            });
        }
        self.cc
    }

    fn emit(&mut self, kind: TokenKind, value: String) {
        self.queue
            .push_back(Token::new(kind, value, self.cur_pos()))
    }

    fn emit_eof(&mut self) {
        let pos = Position::new(self.line, self.column + 1);
        self.queue
            .push_back(Token::new(TokenKind::Eof, "".to_string(), pos));
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
            self.buf_pos = self.cur_pos();
        }
        self.buf.push(c);
    }

    fn cur_pos(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn err<T>(&self, kind: ErrorKind) -> Result<T, SyntaxError> {
        Err(SyntaxError::new(
            kind,
            self.cur_pos(),
            self.filename.clone(),
        ))
    }

    fn err_at(&self, kind: ErrorKind, pos: Position) -> Result<(), SyntaxError> {
        Err(SyntaxError::new(kind, pos, self.filename.clone()))
    }

    fn err_eof(&self) -> Result<(), SyntaxError> {
        self.err_at(ErrorKind::Eof, Position::new(self.line, self.column + 1))
    }

    fn unterminated_single_quote(&mut self) -> Result<(), SyntaxError> {
        self.err_at(
            ErrorKind::UnterminatedSingleQuotedString,
            self.single_quote_pos,
        )
    }

    fn unterminated_double_quote(&mut self) -> Result<(), SyntaxError> {
        let pos = self.quoting_stack.pop_back().unwrap();
        self.err_at(ErrorKind::UnterminatedDoubleQuotedString, pos)
    }

    fn unterminated_expansion(&mut self) -> Result<(), SyntaxError> {
        let pos = self.expansion_stack.pop_back().unwrap();
        self.err_at(ErrorKind::UnterminatedExpansion, pos)
    }
}
