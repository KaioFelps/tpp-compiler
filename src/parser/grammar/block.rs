use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub fn parse_block(&mut self) -> GrammarFinalParseResult<'this> {
        // bloco ::= ( acao ( FIM_DE_SENTENCA bloco )? )?
        let mut block = CstNode::new_non_terminal(NonTerminal::Block);

        match self.maybe_parse_action()? {
            Some(action) => block.append_child(action),
            None => return Ok(block),
        };

        let has_following_block = self.peek_current().is_of_kind(TokenKind::FimDeSentenca);
        if !has_following_block {
            return Ok(block);
        }

        let eos = CstNode::new_terminal(self.expect_token(|| block.get_last_token())?);
        block.append_child(eos);

        let inner_block = self.parse_block()?;
        block.append_child(inner_block);

        Ok(block)
    }
}
