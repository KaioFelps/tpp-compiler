use log::debug;

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResultWithLookahead, NonTerminal};
use crate::token::{CanBeOfKind, IntoKind, TokenKind};

impl<'this> Parser<'this> {
    /// Performs a lookahead to decide via `First(operador_logico)` whether there can be a logical operator
    pub(super) fn maybe_parse_logical_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_logical_operator."#);
        self.parse_as_non_term_if_matches(NonTerminal::LogicalOperator, &[TokenKind::E, TokenKind::Ou])
            .inspect(|node| {
                self.maybe_emit_attrib_shortcut_warning(node, "WAR-SYN-TRIED-TO-USE-LOGICAL-OP-AND-SET-SHORTCUT")
            })
    }

    /// Performs a lookahead to decide via `First(operador_relacional)` whether there can be a relational operator
    pub(super) fn maybe_parse_relational_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_relational_operator."#);
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
        .inspect(|node| {
            let just_parsed_equal = node
                .as_ref()
                .is_some_and(|node| node.get_last_token().is_of_kind(TokenKind::Igual));

            if just_parsed_equal
                && let Some(next_token_to_be_parsed) = self.peek_current()
                && next_token_to_be_parsed.is_of_kind(TokenKind::Igual)
            {
                self.emit_error(
                    self.generate_error_with_token("WAR-SYN-WRONG-EQUAL-OPERATOR", next_token_to_be_parsed),
                );
                // bump it
                let _ = self.take_current();
            }
        })
    }

    /// Performs a lookahead to decide via `First(operador_soma)` whether there can be a addition operator
    pub(super) fn maybe_parse_addition_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_addition_operator."#);
        self.parse_as_non_term_if_matches(NonTerminal::SumOperator, &[TokenKind::Mais, TokenKind::Menos])
            .inspect(|node| self.maybe_emit_attrib_shortcut_warning(node, "WAR-SYN-TRIED-TO-USE-ADD-AND-SET-SHORTCUT"))
    }

    /// Performs a lookahead to decide via `First(operador_multiplicacao)` whether there can be a multiplication operator
    pub(super) fn maybe_parse_multiplication_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_multiplication_operator."#);
        self.parse_as_non_term_if_matches(NonTerminal::ProductOperator, &[TokenKind::Vezes, TokenKind::Divide])
            .inspect(|node| self.maybe_emit_attrib_shortcut_warning(node, "WAR-SYN-TRIED-TO-USE-MUL-AND-SET-SHORTCUT"))
    }

    /// Performs a lookahead to decide via `First(operador_nao)` whether there can be a not operator
    pub(super) fn maybe_parse_not_operator(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_not_operator."#);
        self.parse_as_non_term_if_matches(NonTerminal::NotOperator, &[TokenKind::Nao])
    }

    fn maybe_emit_attrib_shortcut_warning(&mut self, node: &Option<CstNode<'_>>, warning_key: &str) {
        if node.is_some()
            && let Some(next_token_to_be_parsed) = self.peek_current()
            && next_token_to_be_parsed.is_of_kind(TokenKind::Igual)
        {
            self.emit_error(self.generate_error_with_token(warning_key, next_token_to_be_parsed));
        }
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
