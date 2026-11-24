use itertools::Either::{Left, Right};

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarInnerParseFailable, NonTerminal};
use crate::token::IntoKind;
use crate::token::TokenKind::Virgula;

impl<'this> Parser<'this> {
    pub(super) fn inner_parse_as_tuple(&mut self, expr_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        let mut tuple_node = CstNode::new_non_terminal(NonTerminal::Tuple);

        let is_tuple = self.peek_current().into_kind().is_some_and(|kind| kind == Virgula);
        if !is_tuple {
            return Ok(Right(expr_node));
        }

        let comma = CstNode::new_terminal(self.expect_token(|| expr_node.get_token())?);
        tuple_node.append_child(expr_node);
        tuple_node.append_child(comma);

        while self.lookahead_is_unary_expr() {
            tuple_node.append_child(self.parse_expr()?);
            let comma = CstNode::new_terminal(self.expect_token(|| tuple_node.get_last_token())?);
            tuple_node.append_child(comma);
        }

        Ok(Left(tuple_node))
    }
}
