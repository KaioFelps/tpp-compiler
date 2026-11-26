use log::debug;

use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::GrammarFinalParseResult;
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    pub fn parse_program(&mut self) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_program."#);
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

            if let Err(err) = self.get_token_and_append_if_kind_or_error(
                &mut program_node,
                TokenKind::FimDeSentenca,
                "ERR-SYN-EXPECTED-EOS",
            ) {
                self.emit_error(err);
                self.synchronize_program();
            };
        }

        Ok(program_node)
    }

    fn synchronize_program(&mut self) {
        debug!(r#"[Parser] Chamou synchronize_program."#);

        // must be global because we're parsing at the program level right now, which is the global scope (First(declaracao))
        // however it cannot be the whole First(declaracao) because some of the tokens from this set might appear
        // inside expressions, and at `synchronize_program` we're just recovering from a broken declaration, what means
        // the excluded tokens (Ident and AbreParentese) could be the rest of some broken statement that could ruin this
        // recovery point
        const TOKENS_THAT_INITIATES_GLOBAL_DECLARATIONS: &[TokenKind] = &[TokenKind::Inteiro, TokenKind::Flutuante];

        while let Some(token) = self.peek_current() {
            if token.is_of_kind(TokenKind::FimDeSentenca) {
                if let Some(bumped) = self.take_current() {
                    debug!(r#"[Parser] Bumpou {bumped}"#);
                }
                return;
            }

            if token.is_any_of_kinds(TOKENS_THAT_INITIATES_GLOBAL_DECLARATIONS) {
                return;
            }

            if let Some(bumped) = self.take_current() {
                debug!(r#"[Parser] Bumpou {bumped}"#);
            }
        }
    }
}
