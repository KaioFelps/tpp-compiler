use itertools::Either::{Left, Right};

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarFinalParseResultWithLookahead, NonTerminal};
use crate::token::TokenKind::{Escreva, Flutuante, Inteiro, Leia, Repita, Retorna, Se};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub(super) fn maybe_parse_action(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        // First(action)    = First(expressao) ⋃ First(declaracao_variaveis) ⋃ First(se) ⋃ First(repita) ⋃ First(leia) ⋃ First(escreva) ⋃ First(retorna) ⋃ First(erro)
        //                  = First(expressao_unaria) ⋃ First(tipo) ⋃ { SE, REPITA, LEIA, ESCREVA, RETORNA, ERRO }
        //                  = First(expressao_unaria) ⋃ { INTEIRO, Flutuante, SE, REPITA, LEIA, ESCREVA, RETORNA, ERRO } - { ERRO } (* esse erro parece não existir *)
        let action_first_set = &[Inteiro, Flutuante, Se, Repita, Leia, Escreva, Retorna];
        let is_action = self.lookahead_is_unary_expr() || self.peek_current().is_any_of_kinds(action_first_set);

        if !is_action {
            return Ok(None);
        }

        self.parse_action().map(Some)
    }

    pub(super) fn parse_action(&mut self) -> GrammarFinalParseResult<'this> {
        let mut action = CstNode::new_non_terminal(NonTerminal::Action);

        if let Some(simple_action) = self
            .maybe_parse_se()
            .transpose()
            .or_else(|| self.maybe_parse_repita().transpose())
            .or_else(|| self.maybe_parse_leia().transpose())
            .or_else(|| self.maybe_parse_escreva().transpose())
            .or_else(|| self.maybe_parse_retorna().transpose())
            .transpose()?
        {
            return Ok(action.with_child(simple_action));
        };

        if let Some(var_decl) = self.maybe_parse_var_decl()? {
            return Ok(action.with_child(var_decl));
        }

        let expr = self.parse_expr()?;
        match self.inner_parse_var_attribution(expr)? {
            Left(var_attribution) => action.append_child(var_attribution),
            Right(expr) => action.append_child(expr),
        };

        Ok(action)
    }

    fn maybe_parse_se(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        if !self.peek_current().is_of_kind(TokenKind::Se) {
            return Ok(None);
        }

        let mut if_node = CstNode::new_non_terminal(NonTerminal::If);

        let if_token = CstNode::new_terminal(self.expect_token(|| None)?);
        if_node.append_child(if_token);

        let expr = self.parse_expr()?;
        if_node.append_child(expr);

        let else_token = self.expect_token(|| if_node.get_last_token())?;
        if !else_token.is_of_kind(TokenKind::Entao) {
            return Err(self.generate_error_with_ctx_token(
                "ERR-SYN-IF-MISSING-THEN",
                &else_token,
                if_node.get_last_token(),
            ));
        }
        if_node.append_child(CstNode::new_terminal(else_token));

        let block = self.parse_block()?;
        if_node.append_child(block);

        if self.peek_current().is_of_kind(TokenKind::Senao) {
            let else_node = CstNode::new_terminal(self.expect_token(|| if_node.get_last_token())?);
            let block = self.parse_block()?;
            if_node.append_child(else_node);
            if_node.append_child(block);
        }

        let end = self.expect_token(|| if_node.get_last_token())?;
        if !end.is_of_kind(TokenKind::Fim) {
            return Err(self.generate_error_with_ctx_token("ERR-SYN-IF-MISSING-END", &end, if_node.get_last_token()));
        }
        let end = CstNode::new_terminal(end);
        if_node.append_child(end);

        Ok(Some(if_node))
    }

    fn maybe_parse_repita(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        let is_repeat = self.peek_current().is_of_kind(TokenKind::Repita);
        if !is_repeat {
            return Ok(None);
        }

        let mut repeat = CstNode::new_non_terminal(NonTerminal::Loop);
        let repeat_term_node = CstNode::new_terminal(self.expect_token(|| None)?);
        repeat.append_child(repeat_term_node);

        repeat.append_child(self.parse_block()?);

        let until = self.expect_token(|| repeat.get_last_token())?;
        if !until.is_of_kind(TokenKind::Ate) {
            const ERROR_KEY: &str = "ERR-SYN-REPEAT-MISSING-STOP-CLAUSE";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &until, repeat.get_last_token()));
        }
        let until = CstNode::new_terminal(until);
        repeat.append_child(until);

        repeat.append_child(self.parse_expr()?);

        Ok(Some(repeat))
    }

    fn maybe_parse_leia(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        let is_read_stmt = self.peek_current().is_of_kind(TokenKind::Leia);
        if !is_read_stmt {
            return Ok(None);
        }

        let mut read_cmd = CstNode::new_non_terminal(NonTerminal::Read);
        let read_term_node = CstNode::new_terminal(self.expect_token(|| None)?);
        read_cmd.append_child(read_term_node);

        let paren_open = self.expect_token(|| read_cmd.get_last_token())?;
        if !paren_open.is_of_kind(TokenKind::AbreParentese) {
            const ERROR_KEY: &str = "ERR-SYN-LEIA-MISSING-PAREN-OPEN";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_open, read_cmd.get_last_token()));
        }
        read_cmd.append_child(CstNode::new_terminal(paren_open));
        read_cmd.append_child(self.parse_var()?);

        let paren_close = self.expect_token(|| read_cmd.get_last_token())?;
        if !paren_close.is_of_kind(TokenKind::FechaParentese) {
            const ERROR_KEY: &str = "ERR-SYN-LEIA-MISSING-PAREN-CLOSE";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_close, read_cmd.get_last_token()));
        }
        read_cmd.append_child(CstNode::new_terminal(paren_close));

        Ok(Some(read_cmd))
    }

    fn maybe_parse_escreva(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        let is_write_stmt = self.peek_current().is_of_kind(TokenKind::Escreva);
        if !is_write_stmt {
            return Ok(None);
        }

        let mut write_cmd = CstNode::new_non_terminal(NonTerminal::Write);
        let write_term_node = CstNode::new_terminal(self.expect_token(|| None)?);
        write_cmd.append_child(write_term_node);

        let paren_open = self.expect_token(|| write_cmd.get_last_token())?;
        if !paren_open.is_of_kind(TokenKind::AbreParentese) {
            const ERROR_KEY: &str = "ERR-SYN-ESCREVA-MISSING-PAREN-OPEN";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_open, write_cmd.get_last_token()));
        }
        write_cmd.append_child(CstNode::new_terminal(paren_open));
        write_cmd.append_child(self.parse_expr()?);

        let paren_close = self.expect_token(|| write_cmd.get_last_token())?;
        if !paren_close.is_of_kind(TokenKind::FechaParentese) {
            const ERROR_KEY: &str = "ERR-SYN-ESCREVA-MISSING-PAREN-CLOSE";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_close, write_cmd.get_last_token()));
        }
        write_cmd.append_child(CstNode::new_terminal(paren_close));

        Ok(Some(write_cmd))
    }

    fn maybe_parse_retorna(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        let is_return_stmt = self.peek_current().is_of_kind(TokenKind::Retorna);
        if !is_return_stmt {
            return Ok(None);
        }

        let mut return_cmd = CstNode::new_non_terminal(NonTerminal::Return);
        let return_term_node = CstNode::new_terminal(self.expect_token(|| None)?);
        return_cmd.append_child(return_term_node);

        let paren_open = self.expect_token(|| return_cmd.get_last_token())?;
        if !paren_open.is_of_kind(TokenKind::AbreParentese) {
            const ERROR_KEY: &str = "ERR-SYN-RETORNA-MISSING-PAREN-OPEN";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_open, return_cmd.get_last_token()));
        }
        return_cmd.append_child(CstNode::new_terminal(paren_open));
        return_cmd.append_child(self.parse_expr_list()?);

        let paren_close = self.expect_token(|| return_cmd.get_last_token())?;
        if !paren_close.is_of_kind(TokenKind::FechaParentese) {
            const ERROR_KEY: &str = "ERR-SYN-RETORNA-MISSING-PAREN-CLOSE";
            return Err(self.generate_error_with_ctx_token(ERROR_KEY, &paren_close, return_cmd.get_last_token()));
        }
        return_cmd.append_child(CstNode::new_terminal(paren_close));

        Ok(Some(return_cmd))
    }
}
