use std::collections::VecDeque;

use crate::{
    errors::{TppCompilerError, TppCompilerErrorsBag},
    token::{CanBeOfKind, IntoOnwedToken, Token, TokenKind},
};

mod cst;
mod grammar;

pub use cst::*;

pub struct Parser<'this> {
    tokens: VecDeque<Token<'this>>,
    file: &'this str,
    /// Whether the errors should have `force_key` option enabled.
    force_error_key: bool,
    errors_bag: TppCompilerErrorsBag,
}

#[derive(Debug)]
pub enum ParserOutput<'a> {
    Success(CstNode<'a>),
    PartialFailure(CstNode<'a>, TppCompilerErrorsBag),
    Failure(TppCompilerErrorsBag),
}

impl<'this> Parser<'this> {
    fn generate_error(&self, key: &str) -> TppCompilerError {
        TppCompilerError::new_parser(key, self.force_error_key, self.file.to_string(), None, None)
    }

    fn generate_error_with_token(&self, key: &str, current_token: &Token<'this>) -> TppCompilerError {
        TppCompilerError::new_parser(
            key,
            self.force_error_key,
            self.file.to_string(),
            Some(current_token.make_owned()),
            None,
        )
    }

    fn generate_error_with_token_opt(&self, key: &str, current_token: Option<&Token<'this>>) -> TppCompilerError {
        TppCompilerError::new_parser(
            key,
            self.force_error_key,
            self.file.to_string(),
            current_token.map(Token::make_owned),
            None,
        )
    }

    fn generate_error_with_ctx_token<Current: IntoOnwedToken, Extra: IntoOnwedToken>(
        &self,
        key: &str,
        current_token: Option<Current>,
        extra_token: Option<Extra>,
    ) -> TppCompilerError {
        TppCompilerError::new_parser(
            key,
            self.force_error_key,
            self.file.to_string(),
            current_token.map(|token| token.make_owned()),
            extra_token.map(|token| token.make_owned()),
        )
    }

    fn generate_eof_error(&self, current_token: Option<&Token<'_>>) -> TppCompilerError {
        let rich_message_key = if !self.force_error_key && current_token.is_some() {
            "ERR-SYN-EOF-INESPERADO-LOCALIZADO"
        } else {
            "ERR-SYN-EOF-INESPERADO"
        };

        TppCompilerError::new_parser(
            rich_message_key,
            self.force_error_key,
            self.file.to_string(),
            current_token.map(Token::make_owned),
            None,
        )
    }

    pub fn new(source_file: &'this str, tokens: Vec<Token<'this>>, force_error_key: bool) -> Self {
        Self {
            tokens: VecDeque::from_iter(tokens),
            file: source_file,
            force_error_key,
            errors_bag: TppCompilerErrorsBag::new(),
        }
    }

    pub fn into_cst(mut self) -> ParserOutput<'this> {
        match self.parse_program() {
            Ok(root) if self.errors_bag.is_empty() => ParserOutput::Success(root),
            Ok(root) => ParserOutput::PartialFailure(root, self.errors_bag),
            Err(err) => {
                self.errors_bag.append_error(err);
                ParserOutput::Failure(self.errors_bag)
            }
        }
    }

    fn take_current(&mut self) -> Option<Token<'this>> {
        self.tokens.pop_front()
    }

    fn peek_current(&self) -> Option<&Token<'this>> {
        self.tokens.front()
    }

    /// Expects that the current token is `Some(token)` and returns either
    /// the token or an error of unexpected EOF.
    ///
    /// ## Note
    /// This method lazily executes `nearest_token_fn`, hence you can use
    /// expensive callbacks since they're being called only upon an error.
    fn expect_token<'a, F>(&mut self, nearest_token_fn: F) -> Result<Token<'this>, TppCompilerError>
    where
        F: FnOnce() -> Option<&'a Token<'this>>,
        'this: 'a,
    {
        match self.take_current() {
            Some(token) => Ok(token),
            None => Err(self.generate_eof_error(nearest_token_fn())),
        }
    }

    fn emit_error(&mut self, error: TppCompilerError) {
        self.errors_bag.append_error(error);
    }

    #[inline]
    fn get_token_and_append_if_kind_or_error(
        &mut self,
        subtree_root: &mut CstNode<'this>,
        kind: TokenKind,
        error_key: &str,
    ) -> Result<(), TppCompilerError> {
        if let Some(token) = self.peek_current()
            && !token.is_of_kind(kind)
        {
            return Err(self.generate_error_with_ctx_token(error_key, Some(token), subtree_root.get_last_token()));
        }

        let node = CstNode::new_terminal(self.expect_token(|| subtree_root.get_last_token())?);
        subtree_root.append_child(node);
        Ok(())
    }

    #[inline]
    fn get_token_and_append_if_kind_or_error_with_eof_error(
        &mut self,
        subtree_root: &mut CstNode<'this>,
        kind: TokenKind,
        error_key: &str,
        eof_error_key: &str,
    ) -> Result<(), TppCompilerError> {
        if let Some(token) = self.peek_current()
            && !token.is_of_kind(kind)
        {
            return Err(self.generate_error_with_ctx_token(error_key, Some(token), subtree_root.get_last_token()));
        }

        let Some(token) = self.take_current() else {
            return Err(self.generate_error_with_token_opt(eof_error_key, subtree_root.get_last_token()));
        };

        subtree_root.append_child(CstNode::new_terminal(token));
        Ok(())
    }
}
