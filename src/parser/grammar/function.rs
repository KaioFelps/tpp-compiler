use itertools::Either::{Left, Right};

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, GrammarInnerParseFailable, NonTerminal};
use crate::token::Token;

impl<'this> Parser<'this> {
    pub(super) fn parse_return_type(&mut self, token: Token<'this>) -> GrammarFinalParseResult<'this> {
        todo!()
    }

    pub(super) fn parse_func_header(&mut self) -> GrammarFinalParseResult<'this> {
        // ERR-SYN-CABECALHO
        todo!()
    }

    pub(super) fn inner_parse_func_decl_without_return_type(
        &mut self,
        ident_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        todo!()
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

    pub(super) fn inner_parse_func_header(&mut self, ident_node: CstNode<'this>) -> GrammarInnerParseFailable<'this> {
        // ERR-SYN-CABECALHO
        todo!()
    }

    pub(super) fn inner_parse_func_decl_with_tuple_return_type(
        &mut self,
        parenthesis_node: CstNode<'this>,
    ) -> GrammarInnerParseFailable<'this> {
        todo!()
    }
}
