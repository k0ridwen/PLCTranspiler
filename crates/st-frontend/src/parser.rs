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
//! Parser for IEC 61131-3 Structured Text

use std::fmt;

use crate::ast::{Span, Stmt};
use crate::lexer::{Token, TokenType};
use crate::lexer::TokenType::{Eof, Keyword};

/// parser error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    opened: Vec<(String, Span)>,
}

fn is_keyword(tok: &Token, kw: &str) -> bool {
    matches!(tok.token_type, TokenType::Keyword) && tok.value.eq_ignore_ascii_case(kw)
}

fn is_operator(tok: &Token, op: &str) -> bool {
    matches!(tok.token_type, TokenType::Operator) && tok.value == op
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        let (line, col) = tokens.last().map(|t| (t.line, t.col)).unwrap_or((1, 1));
        tokens.push(Token {
            token_type: Eof,
            value: String::new(),
            pos: tokens.len(),
            line,
            col,
        });
        Parser { tokens, pos: 0, opened: Vec::new() }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek2(&self) -> &Token {
        &self.tokens[self.pos + 1]
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        matches!(self.peek().token_type, Keyword) && self.peek().value.eq_ignore_ascii_case(keyword)
    }

    fn check_operator(&self, operator: &str) -> bool {
        matches!(self.peek().token_type, TokenType::Operator) && self.peek().value == operator
    }

    fn advance(&mut self) -> Token {
        self.pos += 1;
        if self.pos <= self.tokens.len() {
            return self.tokens[self.pos].clone();
        }
        let (line, col) = self.tokens.last().map(|t| (t.line, t.col)).unwrap_or((1, 1));
        Token {
            token_type: Eof,
            value: String::new(),
            pos: self.pos,
            line,
            col,
        }
    }

    fn consume_if(&mut self, pred: impl Fn(&Token) -> bool) -> Option<Token> {
        if pred(self.peek()) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> Option<Token> {
        self.consume_if(|tok| is_keyword(tok, keyword))
    }

    fn consume_operator(&mut self, op: &str) -> Option<Token> {
        self.consume_if(|tok| is_operator(tok, op))
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<Token, ParseError> {

    }

    fn expect_operator(&mut self, operator: &str) -> Result<Token, ParseError> {

    }

    fn expect(&mut self, tt: TokenType, what: &str) -> ParseError {

    }
}

pub fn parse_body(input: &str) -> Result<Vec<Stmt>, ParseError> {

}