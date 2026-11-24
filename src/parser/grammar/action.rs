use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::{GrammarFinalParseResult, GrammarFinalParseResultWithLookahead};
use crate::token::TokenKind::{Escreva, Flutuante, Inteiro, Leia, Repita, Retorna, Se};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub(super) fn maybe_parse_action(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        // First(action)    = First(expressao) ⋃ First(declaracao_variaveis) ⋃ First(se) ⋃ First(repita) ⋃ First(leia) ⋃ First(escreva) ⋃ First(retorna) ⋃ First(erro)
        //                  = First(expressao_unaria) ⋃ First(tipo) ⋃ { SE, REPITA, LEIA, ESCREVA, RETORNA, ERRO }
        //                  = First(expressao_unaria) ⋃ { INTEIRO, Flutuante, SE, REPITA, LEIA, ESCREVA, RETORNA, ERRO }
        let action_first_set = &[Inteiro, Flutuante, Se, Repita, Leia, Escreva, Retorna];
        let is_action = self.lookahead_is_unary_expr() || self.peek_current().is_any_of_kinds(action_first_set);

        if !is_action {
            return Ok(None);
        }

        self.parse_action().map(Some)
    }

    pub(super) fn parse_action(&mut self) -> GrammarFinalParseResult<'this> {
        todo!()
    }
}
