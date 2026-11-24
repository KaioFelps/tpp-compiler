use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarInnerParseFailable, NonTerminal};
use crate::token::{IntoKind, TokenKind};
use itertools::Either::{Left, Right};

impl<'this> Parser<'this> {
    pub(super) fn inner_parse_var_decl(&mut self, type_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        let aint_var_decl = self
            .peek_current()
            .into_kind()
            .is_none_or(|kind| kind != TokenKind::DoisPontos);
        if aint_var_decl {
            return Ok(Right(type_node));
        }

        let mut var_decl_node = CstNode::new_non_terminal(NonTerminal::VarDeclaration);
        var_decl_node.append_child(type_node);
        let last_token = var_decl_node.get_last_token();

        let colon = CstNode::new_terminal(self.expect_token(|| last_token)?);
        var_decl_node.append_child(colon);

        let targets_list_node = self.parse_targets_list()?;
        var_decl_node.append_child(targets_list_node);

        Ok(Left(var_decl_node))
    }

    pub(super) fn parse_targets_list(&mut self) -> GrammarFinalParseResult<'this> {
        let var_node = self.parse_var()?;
        self.inner_parse_targets_list(var_node)?
            .left()
            .ok_or_else(|| self.generate_error("ERR-SYN-TARGETS-LIST-INVALIDO"))
    }

    pub(super) fn inner_parse_targets_list(&mut self, var_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        let mut targets_list_node = CstNode::new_non_terminal(NonTerminal::TargetList);
        targets_list_node.append_child(var_node);

        let is_target_list = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kind == TokenKind::Virgula);

        // targets list can be composed of a single var
        // so we face it as successfully parsed here
        if !is_target_list {
            return Ok(Left(targets_list_node));
        }

        while let Some(token) = self.peek_current() {
            if *token.kind() != TokenKind::Virgula {
                break;
            }

            // no need to verify again its a token since we just verified it above
            let comma_token = self.expect_token(|| targets_list_node.get_last_token())?;
            let comma_node = CstNode::new_terminal(comma_token);
            targets_list_node.append_child(comma_node);

            let var_node = self.parse_var()?;
            targets_list_node.append_child(var_node);
        }

        Ok(Left(targets_list_node))
    }

    pub(super) fn parse_var(&mut self) -> GrammarFinalParseResult<'this> {
        let token = self.expect_token(|| None)?;

        if *token.kind() != TokenKind::Ident {
            return Err(self.generate_error_with_token("ERR-SYN-INVALID-VAR-DEF", &token));
        }

        let ident_node = CstNode::new_terminal(token);

        self.inner_parse_var(ident_node)
    }

    pub(super) fn inner_parse_var(&mut self, ident_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        let mut var_node = CstNode::new_non_terminal(NonTerminal::Var);
        var_node.append_child(ident_node);

        let is_var = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kind == TokenKind::AbreColchete);

        if !is_var {
            return Ok(var_node);
        }

        while let Some(token) = self.peek_current()
            && *token.kind() == TokenKind::AbreColchete
        {
            let bracket_open_token = self.expect_token(|| var_node.get_last_token())?;
            let bracket_open_node = CstNode::new_terminal(bracket_open_token);
            var_node.append_child(bracket_open_node);

            let expr_node = self.parse_expr()?;
            var_node.append_child(expr_node);

            let next_token = self.expect_token(|| var_node.get_last_token())?;
            if *next_token.kind() != TokenKind::FechaColchete {
                return Err(self.generate_error_with_token("ERR-SYN-MISSING-CLOSING-BRACKET", &next_token));
            }

            let bracket_open_node = CstNode::new_terminal(next_token);
            var_node.append_child(bracket_open_node);
        }

        Ok(var_node)
    }

    pub(super) fn inner_parse_vars_init_with_parenthesis(
        &mut self,
        parenthesis_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        // First(_continue_as_inicializacao_variaveis) = { ID } as per diagram
        let is_vars_init = self
            .peek_current()
            .into_kind()
            .is_some_and(|kind| kind == TokenKind::Ident);

        if !is_vars_init {
            return Ok(Right(parenthesis_node));
        }

        let mut vars_init_node = CstNode::new_non_terminal(NonTerminal::VarInitialization);
        vars_init_node.append_child(parenthesis_node);

        let targets_list = self.parse_targets_list()?;
        vars_init_node.append_child(targets_list);

        let parenthesis_close_token = match self.take_current() {
            Some(token) => {
                if *token.kind() != TokenKind::FechaParentese {
                    const ERROR_KEY: &str = "ERR-SYN-VARS-INITIALIZATION-EXPECTED-CLOSING-PAREN";
                    return Err(self.generate_error_with_token(ERROR_KEY, &token));
                }

                token
            }
            None => {
                const ERROR_KEY: &str = "ERR-SYN-VARS-INITIALIZATION-MISSING-CLOSING-PAREN";
                return Err(self.generate_error_with_token_opt(ERROR_KEY, vars_init_node.get_last_token()));
            }
        };
        let parenthesis_close_node = CstNode::new_terminal(parenthesis_close_token);
        vars_init_node.append_child(parenthesis_close_node);

        let attribution_node = self.parse_attribution(&vars_init_node)?;
        vars_init_node.append_child(attribution_node);

        let expr_list = self.parse_expr_list()?;
        vars_init_node.append_child(expr_list);

        Ok(Left(vars_init_node))
    }

    pub(super) fn inner_parse_vars_init(&mut self, ident_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        // First(_continue_as_inicializacao_variaveis) = { ABRE_COLCHETE, VIRGULA, ATRIBUICAO }
        let is_vars_init = self.peek_current().into_kind().is_some_and(|kind| {
            matches!(
                kind,
                TokenKind::AbreColchete | TokenKind::Virgula | TokenKind::Atribuicao
            )
        });

        if !is_vars_init {
            return Ok(Right(ident_node));
        }

        // this var node is not for vars init, but to compose a target list
        let var_node = self.inner_parse_var(ident_node)?;
        let target_list_node = match self.inner_parse_targets_list(var_node)? {
            Left(node) => node,
            Right(ident_node) => {
                return Err(self.generate_error_with_token_opt(
                    "ERR-SYN-VARS-INITIALIZATION-INVALID-TARGET-LIST",
                    ident_node.get_token(),
                ));
            }
        };

        let mut vars_init_node = CstNode::new_non_terminal(NonTerminal::VarInitialization);
        vars_init_node.append_child(target_list_node);

        let attribution_node = self.parse_attribution(&vars_init_node)?;
        vars_init_node.append_child(attribution_node);

        let expr_list_node = self.parse_expr_list()?;
        vars_init_node.append_child(expr_list_node);

        Ok(Left(vars_init_node))
    }

    fn parse_attribution(&mut self, latest_node: &CstNode<'this>) -> Result<CstNode<'this>, TppCompilerError> {
        let token = match self.take_current() {
            Some(token) => token,
            None => {
                return Err(self.generate_error_with_token_opt(
                    "ERR-SYN-VARS-INITIALIZATION-MISSING-ATTRIBUTION",
                    latest_node.get_last_token(),
                ));
            }
        };

        let attribution_token = match *token.kind() {
            TokenKind::Atribuicao => token,
            _ => {
                const ERROR_KEY: &str = "ERR-SYN-VARS-INITIALIZATION-EXPECTED-ATTRIBUTION";
                return Err(self.generate_error_with_token(ERROR_KEY, &token));
            }
        };

        Ok(CstNode::new_terminal(attribution_token))
    }
}
