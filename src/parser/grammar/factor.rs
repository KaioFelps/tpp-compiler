use itertools::Either::{Left, Right};

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::CanBeOfKind;
use crate::token::TokenKind::{AbreParentese, FechaParentese, Ident};

impl<'this> Parser<'this> {
    pub(super) fn parse_factor(&mut self) -> GrammarFinalParseResult<'this> {
        let mut factor = CstNode::new_non_terminal(NonTerminal::Factor);

        if self.peek_current().is_of_kind(AbreParentese) {
            let paren_open = CstNode::new_terminal(self.expect_token(|| factor.get_last_token())?);
            let expr = self.parse_expr()?;

            match self.inner_parse_as_tuple(paren_open, expr)? {
                Left(tuple) => factor.append_child(tuple),
                Right((paren_open, expr)) => {
                    factor.append_child(paren_open);
                    factor.append_child(expr);
                }
            };

            let paren_close_token = self.expect_token(|| factor.get_last_token())?;
            if *paren_close_token.kind() != FechaParentese {
                return Err(self.generate_error_with_token("ERR-SYN-FACTOR-MISSING-PAREN-CLOSE", &paren_close_token));
            }
            let paren_close = CstNode::new_terminal(paren_close_token);
            factor.append_child(paren_close);

            return Ok(factor);
        }

        if self.peek_current().is_of_kind(Ident) {
            let ident = CstNode::new_terminal(self.expect_token(|| factor.get_last_token())?);

            if self.peek_current().is_of_kind(AbreParentese) {
                let func_call = self.inner_parse_as_func_call(ident)?;
                factor.append_child(func_call);
            } else {
                let var = self.inner_parse_var(ident)?;
                factor.append_child(var);
            }

            return Ok(factor);
        }

        if let Some(number) = self.maybe_parse_number()? {
            factor.append_child(number);
            return Ok(factor);
        }

        if let Some(array) = self.maybe_parse_array()? {
            factor.append_child(array);
            return Ok(factor);
        }

        Err(self.generate_error_with_token_opt("ERR-SYN-FATOR", self.peek_current()))
    }
}
