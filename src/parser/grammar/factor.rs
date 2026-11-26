use itertools::Either::{Left, Right};
use log::debug;

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::CanBeOfKind;
use crate::token::TokenKind::{AbreParentese, FechaParentese, Ident};

impl<'this> Parser<'this> {
    pub(super) fn parse_factor(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_factor."#);
        let mut factor = CstNode::new_non_terminal(NonTerminal::Factor);

        if self.peek_current().is_of_kind(AbreParentese) {
            let paren_open = CstNode::new_terminal(self.expect_token(|| factor.get_last_token())?);
            let expr = self.parse_expr()?;

            match self.inner_parse_as_tuple(paren_open, expr)? {
                Left(tuple) => factor.append_child(tuple),
                Right((paren_open, expr)) => {
                    factor.append_child(paren_open);
                    factor.append_child(expr);

                    // in this case it's just a (expr) rathern than a tuple,
                    // so we still must require close parenthesis
                    if let Err(err) = self.get_token_and_append_if_kind_or_error(
                        &mut factor,
                        FechaParentese,
                        "ERR-SYN-FACTOR-MISSING-PAREN-CLOSE",
                    ) {
                        self.emit_error(err);
                    };
                }
            };

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
