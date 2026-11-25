use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::GrammarFinalParseResult;
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub fn parse_program(&mut self) -> GrammarFinalParseResult<'this> {
        let mut program_node = CstNode::new_root();

        if self.peek_current().is_none() {
            return Err(self.generate_error("ERR-SYN-EMPTY-PROGRAM"));
        }

        while !self.tokens.is_empty() {
            // this might generates eof so we don't need to worry about it
            match self.parse_declaration() {
                Ok(node) => program_node.append_child(node),
                Err(err) => {
                    self.emit_error(err);
                    self.synchronize_program();
                    continue;
                }
            };

            let next_token = match self.take_current() {
                Some(token) => token,
                // an EOS token is emitted even if the file does not end with \n.
                None => return Err(self.generate_eof_error(program_node.get_last_token())),
            };

            if *next_token.kind() != TokenKind::FimDeSentenca {
                return Err(self.generate_error_with_token("ERR-SYN-EXPECTED-EOS", &next_token));
            }

            let node = CstNode::new(CstNodeKind::Terminal(next_token));
            program_node.append_child(node);
        }

        Ok(program_node)
    }

    fn synchronize_program(&mut self) {
        // bumps everything until the next end of statement token
        // we don't consume it because the `parse_program` is the one
        // to consume this right after calling this method
        while let Some(tok) = self.take_current() {
            if tok.is_of_kind(TokenKind::FimDeSentenca) {
                return;
            }
        }
    }
}
