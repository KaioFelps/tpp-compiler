use std::collections::VecDeque;

use crate::{errors::TppCompilerError, token::Token};

mod cst;
mod grammar;

pub use cst::*;

pub struct Parser<'this> {
    tokens: VecDeque<Token<'this>>,
    file: &'this str,
    /// Whether the errors should have `force_key` option enabled.
    force_error_key: bool,
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

    fn generate_error_with_ctx_token(
        &self,
        key: &str,
        current_token: &Token<'this>,
        extra_token: &Token<'this>,
    ) -> TppCompilerError {
        TppCompilerError::new_parser(
            key,
            self.force_error_key,
            self.file.to_string(),
            Some(current_token.make_owned()),
            Some(extra_token.make_owned()),
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
        }
    }

    pub fn into_cst(mut self) -> Result<CstNode<'this>, TppCompilerError> {
        self.parse_program()
    }

    fn take_current(&mut self) -> Option<Token<'this>> {
        self.tokens.pop_front()
    }

    fn peek_current(&self) -> Option<&Token<'this>> {
        self.tokens.front()
    }

    fn peek_next(&self) -> Option<&Token<'this>> {
        self.tokens.get(1)
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
}
