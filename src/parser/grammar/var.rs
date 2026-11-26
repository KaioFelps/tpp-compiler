use crate::errors::TppCompilerError;
use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{
    GrammarFinalParseResult, GrammarFinalParseResultWithLookahead, GrammarInnerParseFailable, NonTerminal,
};
use crate::token::{CanBeOfKind as _, IntoKind, TokenKind};
use itertools::Either::{Left, Right};
use log::debug;

impl<'this> Parser<'this> {
    pub(super) fn inner_parse_var_decl(&mut self, type_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        debug!(r#"[Parser] Chamou inner_parse_var_decl."#);
        if !self.peek_current().is_of_kind(TokenKind::DoisPontos) {
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
        debug!(r#"[Parser] Chamou parse_targets_list."#);
        let var_node = self.parse_var()?;
        match self.inner_parse_targets_list(var_node)? {
            Left(node) => Ok(node),
            Right(node) => {
                Err(self.generate_error_with_token_opt("ERR-SYN-TARGETS-LIST-INVALIDO", node.get_last_token()))
            }
        }
    }

    pub(super) fn inner_parse_targets_list(&mut self, var_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        debug!(r#"[Parser] Chamou inner_parse_targets_list."#);
        let mut targets_list_node = CstNode::new_non_terminal(NonTerminal::TargetList);
        targets_list_node.append_child(var_node);

        let is_target_list = self.peek_current().is_of_kind(TokenKind::Virgula);

        // targets list can be composed of a single var
        // so we face it as successfully parsed here
        if !is_target_list {
            return Ok(Left(targets_list_node));
        }

        while let Some(token) = self.peek_current()
            && token.is_of_kind(TokenKind::Virgula)
        {
            match self.expect_token(|| targets_list_node.get_last_token()) {
                Ok(comma_token) => targets_list_node.append_child(CstNode::new_terminal(comma_token)),
                Err(err) => self.emit_error(err),
            };

            match self.parse_var() {
                Ok(var_node) => targets_list_node.append_child(var_node),
                Err(err) => self.emit_error(err),
            };
        }

        Ok(Left(targets_list_node))
    }

    pub(super) fn parse_var(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_var."#);
        let ident_node = self.get_token_if_kind_or_error(TokenKind::Ident, "ERR-SYN-INVALID-VAR-DEF")?;
        self.inner_parse_var(ident_node)
    }

    pub(super) fn inner_parse_var(&mut self, ident_node: CstNode<'this>) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou inner_parse_var."#);
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
        debug!(r#"[Parser] Chamou inner_parse_vars_init_with_parenthesis."#);
        // First(_continue_as_inicializacao_variaveis) = { ID } as per diagram
        let is_vars_init = self.peek_current().is_of_kind(TokenKind::Ident);
        if !is_vars_init {
            return Ok(Right(parenthesis_node));
        }

        let mut vars_init_node = CstNode::new_non_terminal(NonTerminal::VarInitialization);
        vars_init_node.append_child(parenthesis_node);

        match self.parse_targets_list() {
            Ok(targets_list) => vars_init_node.append_child(targets_list),
            Err(err) => {
                self.emit_error(err);
            }
        };

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
        debug!(r#"[Parser] Chamou inner_parse_vars_init."#);
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
                    ident_node.get_last_token(),
                ));
            }
        };

        let mut vars_init_node = CstNode::new_non_terminal(NonTerminal::VarInitialization);
        vars_init_node.append_child(target_list_node);

        let attribution_node = self.parse_attribution(&vars_init_node)?;
        vars_init_node.append_child(attribution_node);

        let can_start_expr = self.peek_current().is_any_of_kinds(&[
            TokenKind::FimDeSentenca,
            TokenKind::Fim,
            TokenKind::Senao,
            TokenKind::Ate,
        ]);

        if !can_start_expr {
            return Err(self.generate_error_with_token_opt(
                "ERR-SYN-VARS-INITIALIZATION-MISSING-VALUE",
                vars_init_node.get_last_token(),
            ));
        }

        let expr_list_node = self.parse_expr_list()?;
        vars_init_node.append_child(expr_list_node);

        Ok(Left(vars_init_node))
    }

    fn parse_attribution(&mut self, latest_node: &CstNode<'this>) -> Result<CstNode<'this>, TppCompilerError> {
        debug!(r#"[Parser] Chamou parse_attribution."#);
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

    /// Checks whether a variable attribution can be parsed (and parses it if so).
    ///
    /// ## Note
    /// An attribution can be either `expression ATTRIBUTION ...` or `expr_list ATTRIBUTION` string (since `expr_list -> expression`).
    /// Hence, we check if it's an expression by checking whether there is an [`TokenKind::Atribuicao`] or if it's an "expr_list" by
    /// checking that there is a comma ([`TokenKind::Virgula`]). In both cases, it **must** be an attribution. Otherwise it **cannot** be a var attribution
    pub(super) fn inner_parse_var_attribution(
        &mut self,
        expr_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        debug!(r#"[Parser] Chamou inner_parse_var_attribution."#);
        let is_var_attribution = self
            .peek_current()
            .is_any_of_kinds(&[TokenKind::Virgula, TokenKind::Atribuicao]);

        if !is_var_attribution {
            return Ok(Right(expr_node));
        }

        let mut var_attribution = CstNode::new_non_terminal(NonTerminal::Attribution);

        let expr_list = self.inner_parse_expr_list(expr_node)?;
        var_attribution.append_child(expr_list);

        // an attribution **must** have at least one attribution, and it can actually have more than one
        // e.g.: `a := b := 10 * c`
        // this ensures that the loop below will run at least once
        let has_at_least_one_attribution = self.peek_current().is_of_kind(TokenKind::Atribuicao);
        if !has_at_least_one_attribution {
            const ERROR_KEY: &str = "ERR-SYN-VAR-ATTRIBUTION-MISSING-ATTRIBUTION";
            return Err(self.generate_error_with_token_opt(ERROR_KEY, self.peek_current()));
        }

        while self.peek_current().is_of_kind(TokenKind::Atribuicao) {
            let attribution_token = self.expect_token(|| var_attribution.get_last_token())?;
            let attribution_terminal = CstNode::new_terminal(attribution_token);
            var_attribution.append_child(attribution_terminal);

            var_attribution.append_child(self.parse_expr_list()?);
        }

        Ok(Left(var_attribution))
    }

    pub(super) fn maybe_parse_var_decl(&mut self) -> GrammarFinalParseResultWithLookahead<'this> {
        debug!(r#"[Parser] Chamou maybe_parse_var_decl."#);
        let type_node = match self.maybe_parse_type()? {
            Some(type_node) => type_node,
            None => return Ok(None),
        };

        match self.inner_parse_var_decl(type_node)? {
            Left(var_decl) => Ok(Some(var_decl)),
            Right(type_node) => {
                const ERROR_KEY: &str = "ERR-SYN-ACTION-INVALID-VAR-DECL";
                Err(self.generate_error_with_token_opt(ERROR_KEY, type_node.get_last_token()))
            }
        }
    }
}
