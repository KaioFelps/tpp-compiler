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

        if let Some(token) = self.peek_current()
            && token.is_of_kind(FechaParentese)
        {
            let err = self.generate_error_with_token("ERR-SYN-EMPTY-TUPLE", token);
            let _ = self.take_current();
            return Err(err);
        }

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

        self.get_token_and_append_if_kind_or_error(
            &mut tuple_node,
            FechaParentese,
            "ERR-SYN-TUPLE-MISSING-PAREN-CLOSE",
        )?;

        Ok(Left(tuple_node))
    }
}
