use itertools::Either::{Left, Right};
use log::debug;

use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarInnerParseFailable, NonTerminal};
use crate::token::TokenKind::{AbreParentese, FechaParentese, Virgula};
use crate::token::{CanBeOfKind, IntoKind, TokenKind};

impl<'this> Parser<'this> {
    // parse_func_decl removed since it's never referenced, whereas its inner variants.

    pub(super) fn parse_func_header(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_func_header."#);
        let ident_node = self.get_token_if_kind_or_error(TokenKind::Ident, "ERR-SYN-CABECALHO")?;
        self.inner_parse_func_header(ident_node)
    }

    fn inner_parse_return_types(&mut self, paren_open_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou inner_parse_return_types."#);
        let mut return_type_node = CstNode::new_non_terminal(NonTerminal::ReturnType);
        return_type_node.append_child(paren_open_node);

        if let Some(token) = self.peek_current()
            && token.is_of_kind(FechaParentese)
        {
            let err = self.generate_error_with_token("ERR-SYN-MULTIPLE-RETURN-TYPES-EMPTY", token);
            self.emit_error(err);
            let token = self.expect_token(|| return_type_node.get_last_token())?;
            return Ok(return_type_node.with_child(CstNode::new_terminal(token)));
        }

        loop {
            let type_node = match self.maybe_parse_type()? {
                Some(token) => token,
                None => {
                    let Some(token) = self.peek_current() else {
                        return Err(self.generate_eof_error(return_type_node.get_last_token()));
                    };

                    let err = self.generate_error_with_token("ERR-SYN-MULTIPLE-RETURN-TYPES-INVALID", token);
                    self.emit_error(err);
                    break;
                }
            };
            return_type_node.append_child(type_node);

            let has_more_return_types = self.peek_current().is_of_kind(TokenKind::Virgula);
            if !has_more_return_types {
                break;
            }

            let comma_token = self.expect_token(|| return_type_node.get_last_token())?;
            let comma_node = CstNode::new_terminal(comma_token);
            return_type_node.append_child(comma_node);
        }

        if let Err(err) = self.get_token_and_append_if_kind_or_error(
            &mut return_type_node,
            TokenKind::FechaParentese,
            "ERR-SYN-MULTIPLE-RETURN-TYPES-EXPECTED-PAREN-CLOSE",
        ) {
            self.emit_error(err);
        }

        Ok(return_type_node)
    }

    pub(super) fn inner_parse_func_decl_without_return_type(
        &mut self,
        ident_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        debug!(r#"[Parser] Chamou inner_parse_func_decl_without_return_type."#);
        // First(_continue_as_declaracao_funcao_sem_retorno) = First(_continue_as_cabecalho) = { ABRE_PARENTESE }
        let is_func_decl_without_return_type = self.peek_current().is_of_kind(TokenKind::AbreParentese);
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
        debug!(r#"[Parser] Chamou inner_parse_func_decl_with_return_type."#);
        let mut func_decl = CstNode::new_non_terminal(NonTerminal::FuncDeclaration);
        let header = self.parse_func_header()?;
        func_decl.append_child(type_node);
        func_decl.append_child(header);
        Ok(Left(func_decl))
    }

    pub(super) fn inner_parse_func_header(&mut self, ident_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou inner_parse_func_header."#);
        let mut func_header_node = CstNode::new_non_terminal(NonTerminal::FuncHeader);
        func_header_node.append_child(ident_node);

        if let Err(err) = self.get_token_and_append_if_kind_or_error(
            &mut func_header_node,
            TokenKind::AbreParentese,
            "ERR-SYN-FUNC-HEADER-MISSING-PAREN-OPEN",
        ) {
            // immediately return if it clearly cannot continue through the function header production
            if self.peek_current().is_none() || self.peek_current().is_of_kind(TokenKind::FimDeSentenca) {
                return Err(err);
            }
            // emit the error and continue parsing it as a function header production otherwise
            self.emit_error(err);
        }

        let parameters = self.inner_parse_func_params_vec()?;
        for parameter in parameters {
            func_header_node.append_child(parameter);
        }

        let did_close_paren = match self.get_token_and_append_if_kind_or_error(
            &mut func_header_node,
            TokenKind::FechaParentese,
            "ERR-SYN-FUNC-HEADER-MISSING-PAREN-CLOSE",
        ) {
            Ok(_) => true,
            Err(err) => {
                self.emit_error(err);
                false
            }
        };

        if did_close_paren {
            if let Err(err) = self.get_token_and_append_if_kind_or_error(
                &mut func_header_node,
                TokenKind::FimDeSentenca,
                "ERR-SYN-CABECALHO",
            ) {
                self.emit_error(err);
            }
        } else if self.peek_current().is_of_kind(TokenKind::FimDeSentenca) {
            let _ = self.take_current();
        }

        let block_node = self.parse_block(&[TokenKind::Fim])?;
        func_header_node.append_child(block_node);

        self.get_token_and_append_if_kind_or_error_with_eof_error(
            &mut func_header_node,
            TokenKind::Fim,
            "ERR-SYN-FUNC-MISSING-END",
            "ERR-SYN-FUNC-MISSING-END",
        )?;

        Ok(func_header_node)
    }

    fn synchronize_func_params(&mut self) {
        // FOLLOW(lista_parametros) = { VIRGULA, FECHA_PARENTESE, FIM_DE_SENTENCA, FIM }
        while !self.peek_current().is_any_of_kinds(&[
            TokenKind::Virgula,
            TokenKind::FechaParentese,
            TokenKind::FimDeSentenca,
            TokenKind::Fim,
        ]) {
            let _ = self.take_current();
        }
    }

    fn inner_parse_func_params_vec(&mut self) -> Result<Vec<CstNode<'this>>, TppCompilerError> {
        debug!(r#"[Parser] Chamou inner_parse_func_params_vec."#);
        const FUNC_PARAMS_DEFAULT_ERROR_KEY: &str = "ERR-SYN-PARAMETRO";
        let mut nodes = Vec::<CstNode<'this>>::new();
        let mut requires_a_new_parameter = false;

        // outer loop to handle invalid parameters declaration
        loop {
            if self.peek_current().is_of_kind(TokenKind::Ident) {
                self.emit_error(
                    self.generate_error_with_token_opt("ERR-SYN-WRONG-PARAM-DECL-IDIOM", self.peek_current()),
                );
                self.synchronize_func_params();

                // we bump the comma so that the next parameter (if exists) can be parsed
                // correctly
                if self.peek_current().is_of_kind(TokenKind::Virgula) {
                    let _ = self.take_current();
                }

                continue;
            }

            while self
                .peek_current()
                // First(tipo) ⋃ { VIRGULA }
                // to handle cases like f(, inteiro: b)
                .is_any_of_kinds(&[TokenKind::Inteiro, TokenKind::Flutuante, TokenKind::Virgula])
                || requires_a_new_parameter
            {
                requires_a_new_parameter = false;

                match self.inner_parse_func_parameter() {
                    Ok(parameter_node) => nodes.push(parameter_node),
                    Err(err) => {
                        self.emit_error(err);
                        self.synchronize_func_params();
                    }
                }

                let has_more_parameters = self.peek_current().is_of_kind(TokenKind::Virgula);
                if !has_more_parameters {
                    break;
                }

                let comma_token = self.expect_token(|| nodes.last().and_then(|node| node.get_last_token()))?;
                let comma_node = CstNode::new_terminal(comma_token);
                nodes.push(comma_node);
                requires_a_new_parameter = true;
            }

            break;
        }

        // don't return but emit the error instead, so that the rest can be parsed by the callee function parser method
        if requires_a_new_parameter {
            self.emit_error(self.generate_error_with_token_opt(FUNC_PARAMS_DEFAULT_ERROR_KEY, self.peek_current()));
        }

        Ok(nodes)
    }

    fn synchronize_array_parameter(&mut self) {
        // bump any token but those that would be expected to be here
        while let Some(token) = self.peek_current()
            && !token.is_any_of_kinds(&[
                TokenKind::FechaColchete,  // would close the array dimension
                TokenKind::Virgula,        // would indicate a following parameter
                TokenKind::FechaParentese, // would indicate the end of the function parameters signature
                TokenKind::FimDeSentenca, // handle cases where both ] and ) is missing like `inteiro func(inteiro: a[<EOS>`
            ])
        {
            let _ = self.take_current();
        }
    }

    fn inner_parse_func_parameter(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_func_parameter."#);
        const FUNC_PARAMS_DEFAULT_ERROR_KEY: &str = "ERR-SYN-PARAMETRO";
        let mut param_node = CstNode::new_non_terminal(NonTerminal::Param);

        let type_node = match self.maybe_parse_type()? {
            Some(node) => node,
            None => return Err(self.generate_error_with_token_opt(FUNC_PARAMS_DEFAULT_ERROR_KEY, self.peek_current())),
        };

        param_node.append_child(type_node);

        if let Err(err) = self.get_token_and_append_if_kind_or_error(
            &mut param_node,
            TokenKind::DoisPontos,
            "ERR-SYN-PARAM-MISSING-COLON",
        ) {
            self.emit_error(err);
        }

        if let Err(err) =
            self.get_token_and_append_if_kind_or_error(&mut param_node, TokenKind::Ident, FUNC_PARAMS_DEFAULT_ERROR_KEY)
        {
            self.emit_error(err);
        }

        loop {
            let is_array = self
                .peek_current()
                .into_kind()
                .is_some_and(|kind| kind == TokenKind::AbreColchete);

            if !is_array {
                break;
            }

            if let Err(err) = self.get_token_and_append_if_kind_or_error(
                &mut param_node,
                TokenKind::AbreColchete,
                FUNC_PARAMS_DEFAULT_ERROR_KEY,
            ) {
                self.emit_error(err);
            }

            if let Err(err) = self.get_token_and_append_if_kind_or_error(
                &mut param_node,
                TokenKind::FechaColchete,
                "ERR-SYN-PARAM-INVALID-ARRAY-DIM",
            ) {
                self.emit_error(err);
                self.synchronize_array_parameter();

                if self.peek_current().is_of_kind(TokenKind::FechaColchete) {
                    let _ = self.take_current();
                }
            }
        }

        Ok(param_node)
    }

    pub(super) fn inner_parse_func_decl_with_tuple_return_type(
        &mut self,
        parenthesis_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        debug!(r#"[Parser] Chamou inner_parse_func_decl_with_tuple_return_type."#);

        // First(_continue_as_tipos_retorno) = First(tipo) = {Inteiro, Flutuante}
        let is_func_decl_with_multiple_return_types =
            self.peek_current()
                .is_any_of_kinds(&[TokenKind::Inteiro, TokenKind::Flutuante, TokenKind::FechaParentese]);

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
        debug!(r#"[Parser] Chamou inner_parse_as_func_call."#);
        let mut func_call = CstNode::new_non_terminal(NonTerminal::FuncCall).with_child(ident_node);

        if let Err(err) = self.get_token_and_append_if_kind_or_error(
            &mut func_call,
            AbreParentese,
            "ERR-SYN-FUNC-CALL-MISSING-PAREN-OPEN",
        ) {
            self.emit_error(err);
        }

        if !self.peek_current().is_of_kind(FechaParentese) {
            func_call.append_child(self.parse_expr()?);

            while self.peek_current().is_of_kind(Virgula) {
                let comma = self.expect_token(|| func_call.get_last_token())?;
                func_call.append_child(CstNode::new_terminal(comma));
                func_call.append_child(self.parse_expr()?);
            }
        }

        self.get_token_and_append_if_kind_or_error(
            &mut func_call,
            FechaParentese,
            "ERR-SYN-FUNC-CALL-MISSING-PAREN-CLOSE",
        )?;
        Ok(func_call)
    }
}
