use crate::{
    errors::TppCompilerError,
    parser::{Parser, cst::CstNode},
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
            return Ok(Some(CstNode::new_terminal(token)));
        }

        Ok(None)
    }
}
