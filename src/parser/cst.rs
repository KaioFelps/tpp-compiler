//! # Concrete Syntax Tree
//! This module contains the structs and other artifacts needed to produce
//! an concrete syntax tree (CST) from [`Token`]s and [`TokenKind`]s.

use crate::{parser::grammar::NonTerminal, token::Token};

#[derive(Debug)]
pub enum CstNodeKind<'this> {
    NonTerminal(NonTerminal),
    Terminal(Token<'this>),
}

#[derive(Debug)]
pub struct CstNode<'this> {
    kind: CstNodeKind<'this>,
    children: Vec<CstNode<'this>>,
}

impl<'this> CstNode<'this> {
    pub fn new(kind: CstNodeKind<'this>) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }

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

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn append_child(&mut self, node: CstNode<'this>) {
        self.children.push(node);
    }

    pub fn get_token(&self) -> Option<&Token<'this>> {
        match &self.kind {
            CstNodeKind::Terminal(token) => Some(token),
            _ => None,
        }
    }

    pub fn get_last_token(&self) -> Option<&Token<'this>> {
        self.children
            .last()
            .map(|node| node.get_last_token())
            .unwrap_or_else(|| self.get_token())
    }
}
