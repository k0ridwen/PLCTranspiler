/* Copyright (c) 2026, Romain Brouard
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
//! Lexer for IEC 61131-3 Structured Text

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    IntLiteral,
    RealLiteral,
    BaseLiteral,   // 2#1010_1011, 16#DEAD_BEEF, 8#777
    TimeLiteral,   // T#5s, TIME#2h30m15s, T#12.5ms
    DateLiteral,   // D#2026-11-17, TOD#12:30:45, DT#...
    StringLiteral,
    Identifier,
    Keyword,
    Operator,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Dot,
    DotDot,
    Semicolon,
    Comma,
    Eof,
    Unknown,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub pos: usize,
    pub line: usize,
    pub col: usize,
}

impl Token {
    fn new(token_type: TokenType, value: impl Into<String>, pos: usize, line: usize, col: usize) -> Self {
        Token { token_type, value: value.into(), pos, line, col }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?}, {:?}, {}:{})", self.token_type, self.value, self.line, self.col)
    }
}

/// lexing error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl LexError {
    fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        LexError { message: message.into(), line, col }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (line {}, column {})", self.message, self.line, self.col)
    }
}

impl std::error::Error for LexError {}

/// keywords
const KEYWORDS: &[&str] = &[
    "IF", "THEN", "ELSIF", "ELSE", "END_IF",
    "CASE", "OF", "END_CASE",
    "FOR", "TO", "BY", "DO", "END_FOR",
    "WHILE", "END_WHILE",
    "REPEAT", "UNTIL", "END_REPEAT",
    "RETURN", "EXIT",
    "PROGRAM", "END_PROGRAM",
    "FUNCTION", "END_FUNCTION",
    "FUNCTION_BLOCK", "END_FUNCTION_BLOCK",
    "VAR", "VAR_INPUT", "VAR_OUTPUT", "VAR_IN_OUT",
    "VAR_GLOBAL", "VAR_TEMP", "END_VAR",
    "STRUCT", "END_STRUCT", "TYPE", "END_TYPE",
    "CONSTANT", "RETAIN", "AT",
    "AND", "OR", "XOR", "NOT", "MOD",
    "TRUE", "FALSE",
    "PROPERTY",
];

fn is_keyword(ident: &str) -> bool {
    KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(ident))
}

/// prefixes such as T#, TIME#...
const TIME_PREFIXES: &[&str] = &["T", "TIME"];
const DATE_PREFIXES: &[&str] = &["D", "DATE", "TOD", "TIME_OF_DAY", "DT", "DATE_AND_TIME"];


pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    /// skip comments, spaces, line jump...
    fn skip_trivia(&mut self) -> Result<(), LexError> {
        loop {
            match self.current() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek() == Some('/') => {
                    while let Some(c) = self.current() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('(') if self.peek() == Some('*') => {
                    let (line, col) = (self.line, self.col);
                    self.advance();
                    self.advance();
                    self.skip_block_comment(line, col)?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn skip_block_comment(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        let mut depth = 1usize;
        while let Some(c) = self.current() {
            if c == '(' && self.peek() == Some('*') {
                depth += 1;
                self.advance();
                self.advance();
            } else if c == '*' && self.peek() == Some(')') {
                depth -= 1;
                self.advance();
                self.advance();
                if depth == 0 {
                    return Ok(());
                }
            } else {
                self.advance();
            }
        }
        Err(LexError::new("unclosed comment block", line, col))
    }

    /// token reading
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_trivia()?;

        let (pos, line, col) = (self.pos, self.line, self.col);

        let Some(c) = self.current() else {
            return Ok(Token::new(TokenType::Eof, "", pos, line, col));
        };

        // numbers
        if c.is_ascii_digit() {
            return self.read_number(pos, line, col);
        }

        // identifiers, keywords, types (T#5s, DATE#...)
        if c.is_ascii_alphabetic() || c == '_' {
            return self.read_identifier_or_typed_literal(pos, line, col);
        }

        // strings
        if c == '\'' {
            return self.read_string(pos, line, col);
        }

        // operators
        self.advance();

        let two = |a: char, b: char| c == a && self.current() == Some(b);

        if two(':', '=') {
            self.advance();
            return Ok(Token::new(TokenType::Operator, ":=", pos, line, col));
        }
        if two('*', '*') {
            self.advance();
            return Ok(Token::new(TokenType::Operator, "**", pos, line, col));
        }
        if two('<', '=') {
            self.advance();
            return Ok(Token::new(TokenType::Operator, "<=", pos, line, col));
        }
        if two('>', '=') {
            self.advance();
            return Ok(Token::new(TokenType::Operator, ">=", pos, line, col));
        }
        if two('<', '>') {
            self.advance();
            return Ok(Token::new(TokenType::Operator, "<>", pos, line, col));
        }
        if two('.', '.') {
            self.advance();
            return Ok(Token::new(TokenType::DotDot, "..", pos, line, col));
        }

        let token_type = match c {
            '+' | '-' | '*' | '/' | '=' | '<' | '>' => TokenType::Operator,
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '[' => TokenType::LBracket,
            ']' => TokenType::RBracket,
            ':' => TokenType::Colon,
            '.' => TokenType::Dot,
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            _ => TokenType::Unknown,
        };

        Ok(Token::new(token_type, c.to_string(), pos, line, col))
    }

    fn read_number(&mut self, pos: usize, line: usize, col: usize) -> Result<Token, LexError> {
        let mut value = String::new();

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.current() == Some('#') {
            self.advance();
            value.push('#');
            while let Some(ch) = self.current() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(Token::new(TokenType::BaseLiteral, value, pos, line, col));
        }

        let mut is_real = false;
        if self.current() == Some('.') && self.peek().is_some_and(|p| p.is_ascii_digit()) {
            is_real = true;
            value.push('.');
            self.advance();
            while let Some(ch) = self.current() {
                if ch.is_ascii_digit() || ch == '_' {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }

            if matches!(self.current(), Some('e' | 'E')) {
                let exp_value = self.peek().is_some_and(|p| p.is_ascii_digit());
                let exp_signed = matches!(self.peek(), Some('+' | '-'));
                if exp_value || exp_signed {
                    value.push(self.advance().unwrap()); // e
                    if matches!(self.current(), Some('+' | '-')) {
                        value.push(self.advance().unwrap());
                    }
                    let mut has_digits = false;
                    while let Some(ch) = self.current() {
                        if ch.is_ascii_digit() {
                            has_digits = true;
                            value.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if !has_digits {
                        return Err(LexError::new("unfinished exponent", line, col));
                    }
                }
            }
        }

        let token_type = if is_real { TokenType::RealLiteral } else { TokenType::IntLiteral };
        Ok(Token::new(token_type, value, pos, line, col))
    }

    fn read_identifier_or_typed_literal(
        &mut self,
        pos: usize,
        line: usize,
        col: usize,
    ) -> Result<Token, LexError> {
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.current() == Some('#') {
            let upper = value.to_ascii_uppercase();
            let prefix_kind = if TIME_PREFIXES.contains(&upper.as_str()) {
                Some(TokenType::TimeLiteral)
            } else if DATE_PREFIXES.contains(&upper.as_str()) {
                Some(TokenType::DateLiteral)
            } else {
                None
            };
            if let Some(token_type) = prefix_kind {
                self.advance(); // #
                value.push('#');
                while let Some(ch) = self.current() {
                    if ch.is_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | ';' | ',') {
                        break;
                    }
                    value.push(ch);
                    self.advance();
                }
                return Ok(Token::new(token_type, value, pos, line, col));
            }
        }

        let token_type = if is_keyword(&value) { TokenType::Keyword } else { TokenType::Identifier };
        Ok(Token::new(token_type, value, pos, line, col))
    }

    fn read_string(&mut self, pos: usize, line: usize, col: usize) -> Result<Token, LexError> {
        self.advance();
        let mut value = String::new();
        loop {
            match self.current() {
                None => return Err(LexError::new("unclosed string", line, col)),
                Some('\'') => {
                    self.advance();
                    break;
                }
                Some('$') => {
                    match self.peek() {
                        Some('$') => {
                            value.push('$');
                            self.advance();
                            self.advance();
                        }
                        Some('\'') => {
                            value.push('\'');
                            self.advance();
                            self.advance();
                        }
                        // $L/$N = end of line, $P = form feed, $T = tab, $R = CR
                        Some(e @ ('L' | 'N' | 'P' | 'T' | 'R' | 'l' | 'n' | 'p' | 't' | 'r')) => {
                            let decoded = match e.to_ascii_uppercase() {
                                'L' | 'N' => '\n',
                                'P' => '\x0C',
                                'T' => '\t',
                                _ => '\r',
                            };
                            value.push(decoded);
                            self.advance();
                            self.advance();
                        }
                        // hexadecimal : $41 -> 'A'
                        Some(h) if h.is_ascii_hexdigit() => {
                            self.advance(); // $
                            let mut hex = String::new();
                            for _ in 0..2 {
                                match self.current() {
                                    Some(d) if d.is_ascii_hexdigit() => {
                                        hex.push(d);
                                        self.advance();
                                    }
                                    _ => return Err(LexError::new(
                                        "incomplete hexadecimal escape ($XX)",
                                        line,
                                        col,
                                    )),
                                }
                            }
                            let byte = u8::from_str_radix(&hex, 16).unwrap();
                            value.push(byte as char);
                        }
                        Some(_) => {
                            value.push('$');
                            self.advance();
                        }
                        None => return Err(LexError::new("unclosed string", line, col)),
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }
        Ok(Token::new(TokenType::StringLiteral, value, pos, line, col))
    }
}

/// tokenise ST source, last token is always Eof.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token()?;
        let is_eof = token.token_type == TokenType::Eof;
        tokens.push(token);
        if is_eof {
            break;
        }
    }
    Ok(tokens)
}

/// debug print
pub fn print_tokens(tokens: &[Token]) {
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {i}: {token}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types(input: &str) -> Vec<TokenType> {
        tokenize(input).unwrap().into_iter().map(|t| t.token_type).collect()
    }

    #[test]
    fn if_block_with_keywords_and_assign() {
        let toks = tokenize("IF Start AND NOT EStop THEN x := TRUE; END_IF").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Keyword);
        assert_eq!(toks[0].value, "IF");
        assert!(toks.iter().any(|t| t.value == ":=" && t.token_type == TokenType::Operator));
        assert_eq!(toks.last().unwrap().token_type, TokenType::Eof);
    }

    #[test]
    fn keywords_are_case_insensitive() {
        let toks = tokenize("if x then y := false; end_if").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Keyword);
        assert_eq!(toks[0].value, "if");
        assert!(toks.iter().any(|t| t.value == "end_if" && t.token_type == TokenType::Keyword));
    }

    #[test]
    fn keyword_boundary_is_not_eaten_by_identifiers() {
        let toks = tokenize("IF_alarm").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Identifier);
        assert_eq!(toks[0].value, "IF_alarm");
    }

    #[test]
    fn time_literals() {
        assert_eq!(types("T#5s"), vec![TokenType::TimeLiteral, TokenType::Eof]);
        assert_eq!(types("time#2h30m15s"), vec![TokenType::TimeLiteral, TokenType::Eof]);
        assert_eq!(types("T#12.5ms"), vec![TokenType::TimeLiteral, TokenType::Eof]);
        let toks = tokenize("T#5s").unwrap();
        assert_eq!(toks[0].value, "T#5s");
    }

    #[test]
    fn date_literals() {
        assert_eq!(types("D#2026-11-17"), vec![TokenType::DateLiteral, TokenType::Eof]);
        assert_eq!(types("TOD#12:30:45"), vec![TokenType::DateLiteral, TokenType::Eof]);
    }

    #[test]
    fn base_literals() {
        let toks = tokenize("16#DEAD_BEEF").unwrap();
        assert_eq!(toks[0].token_type, TokenType::BaseLiteral);
        assert_eq!(toks[0].value, "16#DEAD_BEEF");
        assert_eq!(types("2#1010_1011"), vec![TokenType::BaseLiteral, TokenType::Eof]);
    }

    #[test]
    fn int_vs_case_range_dotdot() {
        let toks = tokenize("1..5").unwrap();
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.token_type).collect();
        assert_eq!(
            kinds,
            vec![TokenType::IntLiteral, TokenType::DotDot, TokenType::IntLiteral, TokenType::Eof]
        );
    }

    #[test]
    fn real_with_exponent() {
        assert_eq!(types("1.5e-3"), vec![TokenType::RealLiteral, TokenType::Eof]);
        assert_eq!(types("3.14"), vec![TokenType::RealLiteral, TokenType::Eof]);
        assert_eq!(types("42"), vec![TokenType::IntLiteral, TokenType::Eof]);
    }

    #[test]
    fn strings_with_escapes() {
        let toks = tokenize("'l''eau'").unwrap();
        let _ = toks;
        let toks = tokenize("'l$'eau$$'").unwrap();
        assert_eq!(toks[0].token_type, TokenType::StringLiteral);
        assert_eq!(toks[0].value, "l'eau$");
        let toks = tokenize("$41").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Unknown);
        let toks = tokenize("'$41'").unwrap();
        assert_eq!(toks[0].value, "A");
    }

    #[test]
    fn nested_block_comments() {
        let toks = tokenize("(* lv 1 (* lv 2 *) lv 1 *) x").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Identifier);
        assert_eq!(toks[0].value, "x");
    }

    #[test]
    fn line_comments() {
        let toks = tokenize("// comment\nx := 1;").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Identifier);
    }

    #[test]
    fn unclosed_comment_is_an_error_with_position() {
        let err = tokenize("(* unclosed").unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 1);
    }

    #[test]
    fn token_positions_are_tracked() {
        let toks = tokenize("IF x\nTHEN y := 1; END_IF").unwrap();
        let then_tok = toks.iter().find(|t| t.value == "THEN").unwrap();
        assert_eq!(then_tok.line, 2);
        assert_eq!(then_tok.col, 1);
    }

    #[test]
    fn direct_address_becomes_unknown_not_a_crash() {
        let toks = tokenize("%IX0.0").unwrap();
        assert_eq!(toks[0].token_type, TokenType::Unknown);
        assert_eq!(toks[0].value, "%");
    }

    #[test]
    fn two_char_operators() {
        assert!(tokenize("a <= b").unwrap().iter().any(|t| t.value == "<="));
        assert!(tokenize("a <> b").unwrap().iter().any(|t| t.value == "<>"));
        assert!(tokenize("a ** 2").unwrap().iter().any(|t| t.value == "**"));
    }
}
