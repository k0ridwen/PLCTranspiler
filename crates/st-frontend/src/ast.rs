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
//! AST for IEC 61131-3 Structured Text

#[derive(Clone, Debug, PartialEq)]
pub enum Literal { Bool(bool), Int(String), Real(String), Base(String), Time(String), Date(String), Str(String) }

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum UnaryOp { Not, Neg }

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum BinaryOp { And, Or, Xor, Add, Sub, Mul, Div, Mod, Pow, Eq, Ne, Lt, Gt, Le, Ge }

#[derive(Clone, Debug, PartialEq)]
pub struct CallArg { pub name: Option<String>, pub value: Expr }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span { pub line: usize, pub col: usize }

#[derive(Debug, Clone, PartialEq)]
pub enum Access {
    Field(String),
    Index(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal { value: Literal, span: Span },
    Ref { base: String, path: Vec<Access>, span: Span },
    Unary { op: UnaryOp, operand: Box<Expr>, span: Span },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr>, span: Span },
    Call { target: String, args: Vec<CallArg>, span: Span },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VarKind { Input, Output, InOut, Global, Temp, Local }

#[derive(Clone, Debug, PartialEq)]
pub enum CaseLabel { Value(Expr), Range(Expr, Expr) }

#[derive(Clone, Debug, PartialEq)]
pub struct CaseArm {
    pub labels: Vec<CaseLabel>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Assign { target: Expr, value: Expr, span: Span },
    CallStmt { call: Expr, span: Span },
    If { cond: Expr, then: Vec<Stmt>, elsif: Vec<(Expr, Vec<Stmt>)>, els: Vec<Stmt>, span: Span },
    Case { selector: Expr, arms: Vec<CaseArm>, has_else: bool, els: Vec<Stmt>, span: Span },
    For { var: String, from: Expr, to: Expr, by: Option<Expr>, body: Vec<Stmt>,  span: Span },
    While { cond: Expr, body: Vec<Stmt>, span: Span },
    Repeat { body: Vec<Stmt>, cond: Expr, span: Span },
    Return { span: Span },
    Exit { span: Span },
    VarBlock { kind: VarKind, decls: Vec<VarDecl>, span: Span },
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDecl { pub names: Vec<String>, pub ty: String, pub span: Span }
