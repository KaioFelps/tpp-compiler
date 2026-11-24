use itertools::Either::Left;

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, InnerParseChain, NonTerminal};
use crate::token::TokenKind;

impl<'this> Parser<'this> {
    pub(super) fn parse_declaration(&mut self) -> GrammarFinalParseResult<'this> {
        let mut decl_node = CstNode::new_non_terminal(NonTerminal::Declaration);

        if let Some(type_node) = self.try_parse_type()? {
            if let Left(inner_decl_node) = self
                .inner_parse_var_decl(type_node)
                .on_right_try(|type_node| self.inner_parse_func_decl_with_return_type(type_node))?
            {
                decl_node.append_child(inner_decl_node);
                return Ok(decl_node);
            }

            return Err(self.generate_error("ERR-SYN-IRRECUPERAVEL"));
        };

        // as this is a final grammar parse step, we can directly take it
        // because it always return an error and no rule can parse this token beyond this one
        let token = self.expect_token(|| decl_node.get_last_token())?;
        let kind = token.kind().clone();
        let first_child_node = CstNode::new_terminal(token);

        let node = match kind {
            TokenKind::Ident => self
                .inner_parse_vars_init(first_child_node)
                .on_right_try(|ident_node| self.inner_parse_func_decl_without_return_type(ident_node))?
                .left()
                .ok_or_else(|| self.generate_error("ERR-SYN-IRRECUPERAVEL"))?,
            TokenKind::AbreParentese => self
                .inner_parse_vars_init_with_parenthesis(first_child_node)
                .on_right_try(|parenthesis_node| self.inner_parse_func_decl_with_tuple_return_type(parenthesis_node))?
                .left()
                .ok_or_else(|| self.generate_error("ERR-SYN-IRRECUPERAVEL"))?,
            _ => return Err(self.generate_error("ERR-SYN-IRRECUPERAVEL")),
        };

        decl_node.append_child(node);
        Ok(decl_node)
    }
}
