use itertools::Either::{self, Left, Right};
use log::debug;

use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::NonTerminal;
use crate::token::CanBeOfKind;
use crate::token::TokenKind::{FechaParentese, Virgula};

impl<'this> Parser<'this> {
    pub(super) fn inner_parse_as_tuple(
        &mut self,
        paren_open: CstNode<'this>,
        expr_node: CstNode<'this>,
    ) -> Result<Either<CstNode<'this>, (CstNode<'this>, CstNode<'this>)>, TppCompilerError> {
        debug!(r#"[Parser] Chamou inner_parse_as_tuple."#);
        if !self.peek_current().is_of_kind(Virgula) {
            return Ok(Right((paren_open, expr_node)));
        }

        let mut tuple_node = CstNode::new_non_terminal(NonTerminal::Tuple);
        tuple_node.append_child(paren_open);
        tuple_node.append_child(expr_node);

        loop {
            let comma = CstNode::new_terminal(self.expect_token(|| tuple_node.get_last_token())?);
            tuple_node.append_child(comma);
            tuple_node.append_child(self.parse_expr()?);

            if !self.peek_current().is_of_kind(Virgula) {
                break;
            }
        }

        let paren_close_token = self.expect_token(|| tuple_node.get_last_token())?;
        if *paren_close_token.kind() != FechaParentese {
            return Err(self.generate_error_with_token("ERR-SYN-TUPLE-MISSING-PAREN-CLOSE", &paren_close_token));
        }
        tuple_node.append_child(CstNode::new_terminal(paren_close_token));

        Ok(Left(tuple_node))
    }
}
