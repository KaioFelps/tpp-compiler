use itertools::Either::{Left, Right};

use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarInnerParseFailable, NonTerminal};
use crate::token::{IntoKind, Token, TokenKind};

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

    pub(super) fn parse_return_type(&mut self) -> GrammarFinalParseResult<'this> {
        let token = self.expect_token(|| None)?;
        if *token.kind() != TokenKind::AbreParentese {
            return Err(self.generate_error_with_token("ERR-SYN-MULTIPLE-RETURN-TYPES-MISSING-PAREN-OPEN", &token));
        }
        let parenthesis_node = CstNode::new_terminal(token);
        self.inner_parse_return_type(parenthesis_node)
    }

    fn inner_parse_return_type(&mut self, paren_open_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
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
            let next_token = self.expect_token(|| return_type_node.get_last_token())?;
            if *next_token.kind() != TokenKind::Virgula {
                break;
            }

            let comma_node = CstNode::new_terminal(next_token);
            return_type_node.append_child(comma_node);
        }

        let next_token = self.expect_token(|| return_type_node.get_last_token())?;
        if *next_token.kind() != TokenKind::FechaParentese {
            const ERROR_KEY: &str = "ERR-SYN-MULTIPLE-RETURN-TYPES-EXPECTED-PAREN-CLOSE";
            return Err(self.generate_error_with_token(ERROR_KEY, &next_token));
        }

        let paren_close_node = CstNode::new_terminal(next_token);
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

        self.inner_parse_func_header(ident_node).map(Left)
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

    fn get_last_token_from_nodes_vec<'a>(nodes: &'a [CstNode<'this>]) -> Option<&'a Token<'this>> {
        nodes.last().and_then(|node| node.get_last_token())
    }

    fn inner_parse_func_params_vec(&mut self) -> Result<Vec<CstNode<'this>>, TppCompilerError> {
        const FUNC_PARAMS_DEFAULT_ERROR_KEY: &str = "ERR-SYN-PARAMETROO";
        let mut nodes = Vec::<CstNode<'this>>::new();

        while let Some(type_node) = self.try_parse_type()? {
            nodes.push(type_node);

            let next_token: Token<'_> = self.expect_token(|| Self::get_last_token_from_nodes_vec(&nodes))?;
            if *next_token.kind() != TokenKind::DoisPontos {
                return Err(self.generate_error_with_token_opt(
                    "FUNC_HEADER_DEFAULT_ERROR_KEY",
                    Self::get_last_token_from_nodes_vec(&nodes),
                ));
            }
            let colon_node: CstNode<'_> = CstNode::new_terminal(next_token);
            nodes.push(colon_node);

            let next_token = self.expect_token(|| Self::get_last_token_from_nodes_vec(&nodes))?;
            if *next_token.kind() != TokenKind::Ident {
                return Err(self.generate_error_with_token_opt(
                    "FUNC_HEADER_DEFAULT_ERROR_KEY",
                    Self::get_last_token_from_nodes_vec(&nodes),
                ));
            }
            let iden_node = CstNode::new_terminal(next_token);
            nodes.push(iden_node);

            'array_dimensions_loop: loop {
                let is_array = self
                    .peek_current()
                    .into_kind()
                    .is_some_and(|kind| kind == TokenKind::AbreColchete);

                if !is_array {
                    break 'array_dimensions_loop;
                }

                let brackets_open_token = self.expect_token(|| Self::get_last_token_from_nodes_vec(&nodes))?;
                let brackets_open_node = CstNode::new_terminal(brackets_open_token);
                nodes.push(brackets_open_node);

                let brackets_close_token = self.expect_token(|| Self::get_last_token_from_nodes_vec(&nodes))?;
                if *brackets_close_token.kind() != TokenKind::FechaColchete {
                    return Err(self.generate_error_with_token(FUNC_PARAMS_DEFAULT_ERROR_KEY, &brackets_close_token));
                }

                let brackets_close_node = CstNode::new_terminal(brackets_close_token);
                nodes.push(brackets_close_node);
            }
        }

        Ok(nodes)
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

        let mut function_node = CstNode::new_non_terminal(NonTerminal::FuncDeclaration);
        let return_types: CstNode<'_> = self.inner_parse_return_type(parenthesis_node)?;
        function_node.append_child(return_types);

        let func_header: CstNode<'_> = self.parse_func_header()?;
        function_node.append_child(func_header);

        Ok(Left(function_node))
    }
}
