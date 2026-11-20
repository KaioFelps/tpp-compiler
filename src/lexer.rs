mod manual_dfa_lexer;

use crate::{errors::TppCompilerError, token::Token};
pub use manual_dfa_lexer::*;

pub trait Lexer<'this> {
    /// Lexes the next token.
    fn next_token(&mut self) -> Option<Result<Token<'this>, TppCompilerError>>;

    /// Consumes the whole input and return the list of every token extracted from it.
    fn tokenize(&mut self) -> Result<Vec<Token<'this>>, TppCompilerError> {
        self.get_tokens().collect()
    }

    /// Gets a tokens iterator streaming over the input.
    fn get_tokens(&mut self) -> Box<dyn Iterator<Item = Result<Token<'this>, TppCompilerError>> + '_> {
        Box::new(std::iter::from_fn(move || self.next_token()))
    }

    /// Calculates the column (1-indexed) of the lexeme from its position in the source text line.
    /// Expects `lexeme_begin_position` to be 0-indexed.
    fn get_column(&self, data: &str, lexeme_begin_position: usize) -> usize
    where
        Self: Sized,
    {
        let begin_line: usize = data[0..lexeme_begin_position].rfind('\n').map(|v| v + 1).unwrap_or(0);
        (lexeme_begin_position - begin_line) + 1
    }
}
