use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub fn parse_block(&mut self) -> GrammarFinalParseResult<'this> {
        // bloco ::= ( acao ( FIM_DE_SENTENCA bloco )? )?
        let mut block = CstNode::new_non_terminal(NonTerminal::Block);

        match self.maybe_parse_action() {
            Ok(Some(action)) => block.append_child(action),
            Ok(None) => return Ok(block),
            Err(err) => {
                self.emit_error(err);
                self.synchronize_block();
            }
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

    pub(super) fn synchronize_block(&mut self) {
        while let Some(tok) = self.peek_current() {
            // consumes/bumps the following tokens until the end of the broken block
            if tok.is_of_kind(TokenKind::FimDeSentenca) {
                let _ = self.take_current();
                break;
            }
            // delimiters that belong to the parent node (e.g., SE, REPITA, function bodies, etc.) must not be bumped
            if tok.is_any_of_kinds(&[TokenKind::Fim, TokenKind::Senao, TokenKind::Ate]) {
                break;
            }
            let _ = self.take_current();
        }
    }
}
