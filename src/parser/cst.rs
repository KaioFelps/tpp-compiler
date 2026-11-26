//! # Concrete Syntax Tree
//! This module contains the structs and other artifacts needed to produce
//! an concrete syntax tree (CST) from [`Token`]s and [`TokenKind`]s.

use crate::{
    parser::grammar::NonTerminal,
    token::{CanBeOfKind, Token, TokenKind},
};

#[derive(Debug)]
pub(super) enum CstNodeKind<'this> {
    NonTerminal(NonTerminal),
    Terminal(Token<'this>),
}

#[derive(Debug)]
pub struct CstNode<'this> {
    pub(super) kind: CstNodeKind<'this>,
    pub(super) children: Vec<CstNode<'this>>,
}

impl<'this> CstNode<'this> {
    pub fn new_terminal(token: Token<'this>) -> Self {
        Self {
            kind: CstNodeKind::Terminal(token),
            children: Vec::new(),
        }
    }

    pub fn new_non_terminal(non_terminal: NonTerminal) -> Self {
        Self {
            kind: CstNodeKind::NonTerminal(non_terminal),
            children: Vec::new(),
        }
    }

    pub fn new_root() -> Self {
        Self {
            kind: CstNodeKind::NonTerminal(NonTerminal::Program),
            children: Vec::new(),
        }
    }

    pub fn append_child(&mut self, node: CstNode<'this>) {
        self.children.push(node);
    }

    /// Gets the rightmost token of its children or its own token if it has no children (i.e., it's a terminal).
    /// When being used to accuse an unexpected EOF error, the returned token is the closest token to the EOF.
    ///
    /// ## Note
    /// This method ignores nodes whose token's kind is [`TokenKind::FimDeSentenca`], since these tokens are
    /// virtual and thus are not visible in the actual source code.
    pub fn get_last_token(&self) -> Option<&Token<'this>> {
        match &self.kind {
            CstNodeKind::Terminal(token) if token.is_of_kind(TokenKind::FimDeSentenca) => None,
            CstNodeKind::Terminal(token) => Some(token),
            CstNodeKind::NonTerminal(_) => self.children.iter().rev().find_map(Self::get_last_token),
        }
    }

    pub fn with_child(mut self, node: CstNode<'this>) -> Self {
        self.children.push(node);
        self
    }
}
