use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::GrammarFinalParseResult;
use crate::token::TokenKind;

impl<'this> Parser<'this> {
    pub fn parse_program(&mut self) -> GrammarFinalParseResult<'this> {
        let mut program_node = CstNode::new_root();
        let mut has_read_at_least_one_sequence = false;

        loop {
            if self.peek_current().is_none() && has_read_at_least_one_sequence {
                break;
            }

            // this might generates eof so we don't need to worry about it
            let node = self.parse_declaration()?;
            program_node.append_child(node);

            let next_token = match self.take_current() {
                Some(token) => token,
                // an EOS token is emitted even if the file does not end with \n.
                None => return Err(self.generate_eof_error(program_node.get_last_token())),
            };

            if *next_token.kind() != TokenKind::FimDeSentenca {
                // TODO: return some more specific error
                return Err(self.generate_error_with_token("ERR-SYN-IRRECUPERAVEL", &next_token));
            }

            let node = CstNode::new(CstNodeKind::Terminal(next_token));
            program_node.append_child(node);

            has_read_at_least_one_sequence = true;
        }

        Ok(program_node)
    }
}
