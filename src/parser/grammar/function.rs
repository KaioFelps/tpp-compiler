use itertools::Either::{Left, Right};

use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarInnerParseFailable, NonTerminal};
use crate::token::{IntoKind, TokenKind};

impl<'this> Parser<'this> {
    // parse_func_decl removed since it's never referenced, whereas its inner variants.

    pub(super) fn parse_func_header(&mut self) -> GrammarFinalParseResult<'this> {
        const FUNC_HEADER_DEFAULT_ERROR_KEY: &str = "ERR-SYN-CABECALHO";
        let ident_token = self.expect_token(|| None)?;
        if *ident_token.kind() != TokenKind::Ident {
            return Err(self.generate_error_with_token(FUNC_HEADER_DEFAULT_ERROR_KEY, &ident_token));
        }
        let ident_node = CstNode::new_terminal(ident_token);
        self.inner_parse_func_header(ident_node)
    }

    fn inner_parse_return_types(&mut self, paren_open_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        let mut return_type_node = CstNode::new_non_terminal(NonTerminal::ReturnType);
        return_type_node.append_child(paren_open_node);

        loop {
            let type_node = match self.try_parse_type()? {
                Some(token) => token,
                None => {
                    return if let Some(token) = self.peek_current() {
                        Err(self.generate_error_with_token("ERR-SYN-MULTIPLE-RETURN-TYPES-EXPECTED-TYPE", token))
                    } else {
                        Err(self.generate_eof_error(return_type_node.get_last_token()))
                    };
                }
            };
            return_type_node.append_child(type_node);

            let has_more_return_types = self
                .peek_current()
                .into_kind()
                .is_some_and(|kind| kind == TokenKind::Virgula);

            if !has_more_return_types {
                break;
            }

            let comma_token = self.expect_token(|| return_type_node.get_last_token())?;
            let comma_node = CstNode::new_terminal(comma_token);
            return_type_node.append_child(comma_node);
        }

        let paren_close_token = self.expect_token(|| return_type_node.get_last_token())?;
        if *paren_close_token.kind() != TokenKind::FechaParentese {
            const ERROR_KEY: &str = "ERR-SYN-MULTIPLE-RETURN-TYPES-EXPECTED-PAREN-CLOSE";
            return Err(self.generate_error_with_token(ERROR_KEY, &paren_close_token));
        }

        let paren_close_node = CstNode::new_terminal(paren_close_token);
        return_type_node.append_child(paren_close_node);

        Ok(return_type_node)
    }

    pub(super) fn inner_parse_func_decl_without_return_type(
        &mut self,
        ident_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        // First(_continue_as_declaracao_funcao_sem_retorno) = First(_continue_as_cabecalho) = { ABRE_PARENTESE }
        let is_func_decl_without_return_type = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kind == TokenKind::AbreParentese);

        if !is_func_decl_without_return_type {
            return Ok(Right(ident_node));
        }

        let header_node = self.inner_parse_func_header(ident_node)?;
        let mut func_decl_node = CstNode::new_non_terminal(NonTerminal::FuncDeclaration);
        func_decl_node.append_child(header_node);

        Ok(Left(func_decl_node))
    }

    pub(super) fn inner_parse_func_decl_with_return_type(
        &mut self,
        type_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        let mut func_decl = CstNode::new_non_terminal(NonTerminal::FuncDeclaration);
        let header = self.parse_func_header()?;
        func_decl.append_child(type_node);
        func_decl.append_child(header);
        Ok(Left(func_decl))
    }

    pub(super) fn inner_parse_func_header(&mut self, ident_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        const FUNC_HEADER_DEFAULT_ERROR_KEY: &str = "ERR-SYN-CABECALHO";
        let mut func_header_node = CstNode::new_non_terminal(NonTerminal::FuncHeader);
        func_header_node.append_child(ident_node);

        let next_token = self.expect_token(|| func_header_node.get_last_token())?;
        if *next_token.kind() != TokenKind::AbreParentese {
            return Err(self.generate_error_with_token(FUNC_HEADER_DEFAULT_ERROR_KEY, &next_token));
        }
        let paren_open_node = CstNode::new_terminal(next_token);
        func_header_node.append_child(paren_open_node);

        let parameters = self.inner_parse_func_params_vec()?;
        for parameter in parameters {
            func_header_node.append_child(parameter);
        }

        let next_token = self.expect_token(|| func_header_node.get_last_token())?;
        if *next_token.kind() != TokenKind::FechaParentese {
            return Err(self.generate_error_with_token(FUNC_HEADER_DEFAULT_ERROR_KEY, &next_token));
        }
        let paren_close_node = CstNode::new_terminal(next_token);
        func_header_node.append_child(paren_close_node);

        let next_token = self.expect_token(|| func_header_node.get_last_token())?;
        if *next_token.kind() != TokenKind::FimDeSentenca {
            return Err(self.generate_error_with_token(FUNC_HEADER_DEFAULT_ERROR_KEY, &next_token));
        }
        let end_of_statement_node = CstNode::new_terminal(next_token);
        func_header_node.append_child(end_of_statement_node);

        let block_node = self.parse_block()?;
        func_header_node.append_child(block_node);

        let next_token = self.expect_token(|| func_header_node.get_last_token())?;
        if *next_token.kind() != TokenKind::Fim {
            return Err(self.generate_error_with_token(FUNC_HEADER_DEFAULT_ERROR_KEY, &next_token));
        }
        let end_node = CstNode::new_terminal(next_token);
        func_header_node.append_child(end_node);

        Ok(func_header_node)
    }

    fn inner_parse_func_params_vec(&mut self) -> Result<Vec<CstNode<'this>>, TppCompilerError> {
        const FUNC_PARAMS_DEFAULT_ERROR_KEY: &str = "ERR-SYN-PARAMETRO";
        let mut nodes = Vec::<CstNode<'this>>::new();
        let mut requires_a_new_parameter = false;

        // First(parameter) = First(tipo) = { Inteiro, Flutuante }
        while self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| matches!(kind, TokenKind::Inteiro | TokenKind::Flutuante))
        {
            requires_a_new_parameter = false;
            let parameter_node = self.parse_func_parameter()?;
            nodes.push(parameter_node);

            let has_more_parameters = self
                .peek_current()
                .into_kind()
                .is_some_and(|kind| kind == TokenKind::Virgula);

            if !has_more_parameters {
                break;
            }

            let comma_token = self.expect_token(|| nodes.last().and_then(|node| node.get_last_token()))?;
            let comma_node = CstNode::new_terminal(comma_token);
            nodes.push(comma_node);
            requires_a_new_parameter = true;
        }

        if requires_a_new_parameter {
            return Err(self.generate_error_with_token_opt(FUNC_PARAMS_DEFAULT_ERROR_KEY, self.peek_current()));
        }

        Ok(nodes)
    }

    fn parse_func_parameter(&mut self) -> GrammarFinalParseResult<'this> {
        const FUNC_PARAMS_DEFAULT_ERROR_KEY: &str = "ERR-SYN-PARAMETRO";

        let type_node = match self.try_parse_type()? {
            Some(node) => node,
            None => return Err(self.generate_error_with_token_opt(FUNC_PARAMS_DEFAULT_ERROR_KEY, self.peek_current())),
        };

        let mut param_node = CstNode::new_non_terminal(NonTerminal::Param);
        param_node.append_child(type_node);

        let colon_token = self.expect_token(|| param_node.get_last_token())?;
        if *colon_token.kind() != TokenKind::DoisPontos {
            return Err(self.generate_error_with_token(FUNC_PARAMS_DEFAULT_ERROR_KEY, &colon_token));
        }
        let colon_node: CstNode<'_> = CstNode::new_terminal(colon_token);
        param_node.append_child(colon_node);

        let iden_token = self.expect_token(|| param_node.get_last_token())?;
        if *iden_token.kind() != TokenKind::Ident {
            return Err(self.generate_error_with_token(FUNC_PARAMS_DEFAULT_ERROR_KEY, &iden_token));
        }
        let iden_node = CstNode::new_terminal(iden_token);
        param_node.append_child(iden_node);

        loop {
            let is_array = self
                .peek_current()
                .into_kind()
                .is_some_and(|kind| kind == TokenKind::AbreColchete);

            if !is_array {
                break;
            }

            let brackets_open_token = self.expect_token(|| None)?;
            let brackets_open_node = CstNode::new_terminal(brackets_open_token);
            param_node.append_child(brackets_open_node);

            let brackets_close_token = self.expect_token(|| param_node.get_last_token())?;
            if *brackets_close_token.kind() != TokenKind::FechaColchete {
                return Err(self.generate_error_with_token(FUNC_PARAMS_DEFAULT_ERROR_KEY, &brackets_close_token));
            }

            let brackets_close_node = CstNode::new_terminal(brackets_close_token);
            param_node.append_child(brackets_close_node);
        }

        Ok(param_node)
    }

    pub(super) fn inner_parse_func_decl_with_tuple_return_type(
        &mut self,
        parenthesis_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        // First(_continue_as_tipos_retorno) = First(tipo) = {Inteiro, Flutuante}
        let is_func_decl_with_multiple_return_types = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| matches!(kind, TokenKind::Inteiro | TokenKind::Flutuante));

        if !is_func_decl_with_multiple_return_types {
            return Ok(Right(parenthesis_node));
        }

        let mut func_decl_node = CstNode::new_non_terminal(NonTerminal::FuncDeclaration);
        let return_types: CstNode<'_> = self.inner_parse_return_types(parenthesis_node)?;
        func_decl_node.append_child(return_types);

        let func_header: CstNode<'_> = self.parse_func_header()?;
        func_decl_node.append_child(func_header);

        Ok(Left(func_decl_node))
    }

    pub(super) fn inner_parse_as_func_call(&mut self, ident_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        todo!()
    }
}
