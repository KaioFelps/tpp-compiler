use crate::{
    errors::TppCompilerError,
    parser::{Parser, cst::CstNode, grammar::NonTerminal},
    token::TokenKind,
};

impl<'this> Parser<'this> {
    /// Tries to parse `token` as a type.
    /// If it fails, then it returns `None` rather than an error.
    ///
    /// ## Errors
    /// It only fails if there is no current token on the queue.
    pub(super) fn parse_type(&mut self) -> Result<Option<CstNode<'this>>, TppCompilerError> {
        if self
            .peek_current()
            .is_some_and(|token| matches!(*token.kind(), TokenKind::Inteiro | TokenKind::Flutuante))
        {
            let token = self.take_current().ok_or_else(|| self.generate_eof_error(None))?;
            let terminal_node = CstNode::new_terminal(token);
            let mut type_non_term_node = CstNode::new_non_terminal(NonTerminal::Type);
            type_non_term_node.append_child(terminal_node);
            return Ok(Some(type_non_term_node));
        }

        Ok(None)
    }
}
