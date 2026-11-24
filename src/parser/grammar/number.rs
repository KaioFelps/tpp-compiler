use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResultWithLookahead, NonTerminal};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub(super) fn maybe_parse_number(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        if self.peek_current().is_any_of_kinds(&[
            TokenKind::NumInteiro,
            TokenKind::NumPontoFlutuante,
            TokenKind::NumNotacaoCientifica,
        ]) {
            let token = self.take_current().ok_or_else(|| self.generate_eof_error(None))?;
            let number = CstNode::new_non_terminal(NonTerminal::Number).with_child(CstNode::new_terminal(token));
            return Ok(Some(number));
        }

        Ok(None)
    }
}
