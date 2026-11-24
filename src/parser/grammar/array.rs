use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::GrammarFinalParseResult;
use crate::token::TokenKind;

impl<'this> Parser<'this> {
    pub fn parse_array(&mut self) -> GrammarFinalParseResult<'this> {
        todo!()
    }
}
