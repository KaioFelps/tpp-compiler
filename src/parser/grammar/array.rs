use log::debug;

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarFinalParseResultWithLookahead, NonTerminal};
use crate::token::TokenKind::AbreChave;
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub(super) fn maybe_parse_array(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_array."#);
        if self.peek_current().is_of_kind(AbreChave) {
            let array = self.parse_array()?;
            return Ok(Some(array));
        }

        Ok(None)
    }

    pub(super) fn parse_array(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_array."#);
        let mut array = CstNode::new_non_terminal(NonTerminal::ArrInitialization);

        let curly_brackets_open = self.expect_token(|| None)?;
        if !curly_brackets_open.is_of_kind(TokenKind::AbreChave) {
            return Err(self.generate_error_with_token("ERR-SYN-ARRAY-PAREN-OPEN", &curly_brackets_open));
        }
        let curly_brackets_open = CstNode::new_terminal(curly_brackets_open);
        array.append_child(curly_brackets_open);

        let exprs = self.parse_expr_list()?;
        array.append_child(exprs);

        let curly_brackets_close = self.expect_token(|| array.get_last_token())?;
        if !curly_brackets_close.is_of_kind(TokenKind::FechaChave) {
            return Err(self.generate_error_with_token("ERR-SYN-ARRAY-PAREN-CLOSE", &curly_brackets_close));
        }
        let curly_brackets_close = CstNode::new_terminal(curly_brackets_close);
        array.append_child(curly_brackets_close);

        Ok(array)
    }
}
