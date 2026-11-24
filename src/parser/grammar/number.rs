use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::{GrammarFinalParseResult, GrammarFinalParseResultWithLookahead};
use crate::token::TokenKind;

impl<'this> Parser<'this> {
    pub fn maybe_parse_number(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        todo!()
    }

    pub fn parse_number(&mut self) -> GrammarFinalParseResult<'this> {
        todo!()
    }
}
