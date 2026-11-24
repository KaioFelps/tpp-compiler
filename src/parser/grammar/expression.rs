use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::{IntoKind, TokenKind};

impl<'this> Parser<'this> {
    pub(super) fn parse_expr_list(&mut self) -> GrammarFinalParseResult<'this> {
        let expr_node = self.parse_expr()?;
        self.inner_parse_expr_list(expr_node)
    }

    pub(super) fn parse_expr(&mut self) -> GrammarFinalParseResult<'this> {
        let mut expr_node = CstNode::new_non_terminal(NonTerminal::Expr);
        let simple_expr = self.parse_simple_expr()?;
        expr_node.append_child(simple_expr);

        // First(expressao_simples) = First(expressao_aditiva) = First(expressao_multiplicativa) = First(expressao_unaria)
        while let Some(logical_op) = self.maybe_parse_logical_operator()? {
            expr_node.append_child(logical_op);
            expr_node.append_child(self.parse_simple_expr()?);
        }

        Ok(expr_node)
    }

    fn parse_simple_expr(&mut self) -> GrammarFinalParseResult<'this> {
        let mut simple_expr_node = CstNode::new_non_terminal(NonTerminal::SimpleExpr);
        let additive_expr = self.parse_additive_expr()?;
        simple_expr_node.append_child(additive_expr);

        // First(expressao_aditiva) = First(expressao_multiplicativa) = First(expressao_unaria)
        while let Some(relational_op) = self.maybe_parse_relational_operator()? {
            simple_expr_node.append_child(relational_op);
            simple_expr_node.append_child(self.parse_additive_expr()?);
        }

        Ok(simple_expr_node)
    }

    fn parse_additive_expr(&mut self) -> GrammarFinalParseResult<'this> {
        let mut additive_expr_node = CstNode::new_non_terminal(NonTerminal::AdditionExpr);
        let additive_expr = self.parse_multiplicative_expr()?;
        additive_expr_node.append_child(additive_expr);

        while let Some(addition_op) = self.maybe_parse_addition_operator()? {
            additive_expr_node.append_child(addition_op);
            additive_expr_node.append_child(self.parse_multiplicative_expr()?);
        }

        Ok(additive_expr_node)
    }

    fn parse_multiplicative_expr(&mut self) -> GrammarFinalParseResult<'this> {
        let mut multiplicative_expr_node = CstNode::new_non_terminal(NonTerminal::ProductExpr);
        let unary_expr = self.parse_unary_expr()?;
        multiplicative_expr_node.append_child(unary_expr);

        while let Some(mult_op) = self.maybe_parse_multiplication_operator()? {
            multiplicative_expr_node.append_child(mult_op);
            multiplicative_expr_node.append_child(self.parse_unary_expr()?);
        }

        Ok(multiplicative_expr_node)
    }

    fn parse_unary_expr(&mut self) -> GrammarFinalParseResult<'this> {
        let mut unary_node = CstNode::new_non_terminal(NonTerminal::UnaryExpr);

        if let Some(op) = self.maybe_parse_addition_operator()? {
            unary_node.append_child(op);
        } else if let Some(op) = self.maybe_parse_not_operator()? {
            unary_node.append_child(op);
        }

        let factor = self.parse_factor()?;
        unary_node.append_child(factor);

        Ok(unary_node)
    }

    fn parse_factor(&mut self) -> GrammarFinalParseResult<'this> {
        todo!()
    }

    pub(super) fn inner_parse_expr_list(&mut self, expr_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        let mut expr_list_node = CstNode::new_non_terminal(super::NonTerminal::ExprList);
        expr_list_node.append_child(expr_node);

        while self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kind == TokenKind::Virgula)
        {
            let comma_token = self.expect_token(|| expr_list_node.get_last_token())?;
            let comma_node = CstNode::new_terminal(comma_token);
            expr_list_node.append_child(comma_node);

            let expr_node = self.parse_expr()?;
            expr_list_node.append_child(expr_node);
        }

        Ok(expr_list_node)
    }
}
