use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::{GrammarFinalParseResult, GrammarFinalParseResultWithLookahead};
use crate::token::TokenKind::AbreChave;
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub fn maybe_parse_array(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        if self.peek_current().is_of_kind(AbreChave) {
            let array = self.parse_array()?;
            return Ok(Some(array));
        }

        Ok(None)
    }

    pub fn parse_array(&mut self) -> GrammarFinalParseResult<'this> {
        todo!()
    }
}
