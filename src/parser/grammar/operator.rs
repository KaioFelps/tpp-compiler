use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResultWithLookahead, NonTerminal};
use crate::token::{IntoKind, TokenKind};

impl<'this> Parser<'this> {
    /// Performs a lookahead to decide via `First(operador_logico)` whether there can be a logical operator
    pub(super) fn maybe_parse_logical_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        self.parse_as_non_term_if_matches(NonTerminal::LogicalOperator, &[TokenKind::E, TokenKind::Ou])
    }

    /// Performs a lookahead to decide via `First(operador_relacional)` whether there can be a relational operator
    pub(super) fn maybe_parse_relational_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        self.parse_as_non_term_if_matches(
            NonTerminal::RelationalOperator,
            &[
                TokenKind::Menor,
                TokenKind::Maior,
                TokenKind::Igual,
                TokenKind::Diferente,
                TokenKind::MenorIgual,
                TokenKind::MaiorIgual,
            ],
        )
    }

    /// Performs a lookahead to decide via `First(operador_soma)` whether there can be a addition operator
    pub(super) fn maybe_parse_addition_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        self.parse_as_non_term_if_matches(NonTerminal::SumOperator, &[TokenKind::Mais, TokenKind::Menos])
    }

    /// Performs a lookahead to decide via `First(operador_multiplicacao)` whether there can be a multiplication operator
    pub(super) fn maybe_parse_multiplication_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        self.parse_as_non_term_if_matches(NonTerminal::ProductOperator, &[TokenKind::Vezes, TokenKind::Divide])
    }

    /// Performs a lookahead to decide via `First(operador_nao)` whether there can be a not operator
    pub(super) fn maybe_parse_not_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        self.parse_as_non_term_if_matches(NonTerminal::NotOperator, &[TokenKind::Nao])
    }

    #[inline]
    fn parse_as_non_term_if_matches(
        &mut self,
        non_term: NonTerminal,
        kinds: &[TokenKind],
    ) -> GrammarFinalParseResultWithLookahead<'this> {
        let matches_op_lookahead = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kinds.contains(&kind));

        if !matches_op_lookahead {
            return Ok(None);
        }

        let token = self.expect_token(|| None)?;
        let operator_node = CstNode::new_non_terminal(non_term).with_child(CstNode::new_terminal(token));
        Ok(Some(operator_node))
    }
}
