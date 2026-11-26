use log::debug;

use crate::parser::Parser;
use crate::parser::cst::CstNode;
use crate::parser::grammar::{GrammarFinalParseResult, NonTerminal};
use crate::token::{CanBeOfKind, TokenKind};

impl<'this> Parser<'this> {
    /// Parses a block of code.
    ///
    /// ## Parameters
    /// - `delimiters_set` - A slice of [`TokenKind`]s that might end a block other than [`TokenKind::FimDeSentenca`].
    ///   it's up to the callee to decide what are these delimiters. For instance: for a funcion body, the other delimiter
    ///   is [`TokenKind::Fim`] because it is the end of the function body in the current grammar. Note that it **does not**
    ///   consume the tokends in `delimiters_set`, which is left for the callee to handle.
    pub fn parse_block(&mut self, delimiters_set: &[TokenKind]) -> GrammarFinalParseResult<'this> {
        debug!(r#"[Parser] Chamou parse_block."#);
        // bloco ::= ( acao ( FIM_DE_SENTENCA bloco )? )?
        let mut block = CstNode::new_non_terminal(NonTerminal::Block);

        match self.maybe_parse_action() {
            Ok(Some(action)) => block.append_child(action),
            Ok(None) => {
                // when the current token is not an action and neither one of the delimiters set expected by the parent,
                // then it can only be an invalid statement/action
                if let Some(token) = self.peek_current()
                    && !token.is_any_of_kinds(delimiters_set)
                {
                    let err = self.generate_error_with_token("ERR-SYN-ACAO", token);
                    self.emit_error(err);
                    self.synchronize_block(delimiters_set);
                    return self.parse_block(delimiters_set);
                }

                return Ok(block);
            }
            Err(err) => {
                self.emit_error(err);
                self.synchronize_block(delimiters_set);
                return self.parse_block(delimiters_set);
            }
        };

        let Some(token) = self.peek_current() else {
            return Ok(block);
        };

        if token.is_of_kind(TokenKind::FimDeSentenca) {
            let eos = CstNode::new_terminal(self.expect_token(|| block.get_last_token())?);
            block.append_child(eos);
        } else if token.is_any_of_kinds(delimiters_set) {
            return Ok(block);
        } else {
            let err = self.generate_error_with_token("ERR-SYN-EXPECTED-EOS", token);
            self.emit_error(err);
            self.synchronize_block(delimiters_set);
        }

        let inner_block = self.parse_block(delimiters_set)?;
        block.append_child(inner_block);

        Ok(block)
    }

    pub(super) fn synchronize_block(&mut self, delimiters_set: &[TokenKind]) {
        debug!(r#"[Parser] Chamou synchronize_block."#);
        while let Some(token) = self.peek_current() {
            debug!(r#"[Parser] Tentando sincronizar bloco. (Lendo {token:?})"#);
            // consumes/bumps the following tokens until the end of the broken block
            if token.is_of_kind(TokenKind::FimDeSentenca) {
                debug!(r#"[Parser] Bumpou {token:?}"#);
                let _ = self.take_current();
                break;
            }
            // delimiters that belong to the parent node (e.g., SE, REPITA, function bodies, etc.) must not be bumped
            if token.is_any_of_kinds(delimiters_set) {
                break;
            }
            let token = self.take_current();
            debug!(r#"[Parser] Bumpou {token:?}"#);
        }
    }
}
