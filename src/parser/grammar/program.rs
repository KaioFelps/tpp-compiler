use crate::parser::Parser;
use crate::parser::cst::{CstNode, CstNodeKind};
use crate::parser::grammar::GrammarFinalParseResult;
use crate::token::TokenKind;

impl<'this> Parser<'this> {
    pub fn parse_program(&mut self) -> GrammarFinalParseResult<'this> {
        let mut program_node = CstNode::new_root();

        loop {
            let node = self.parse_declaration()?;
            program_node.append_child(node);

            if let Some(token) = self.take_current() {
                if *token.kind() == TokenKind::FimDeSentenca {
                    let node = CstNode::new(CstNodeKind::Terminal(token));
                    program_node.append_child(node);
                    continue;
                }

                // TODO: return some more specific error
                return Err(self.generate_error_with_token("ERR-SYN-IRRECUPERAVEL", &token));
            } else {
                return Err(self.generate_eof_error(program_node.get_last_token()));
            }
        }
    }
}
