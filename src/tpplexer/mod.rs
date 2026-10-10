mod manual_dfa_lexer;

use crate::token::Token;
pub use manual_dfa_lexer::*;

pub trait Lexer<'this> {
    /// Lexes the next token.
    fn next_token(&mut self) -> Option<Token<'this>>;

    /// Consumes the whole input and return the list of every token extracted from it.
    fn tokenize(&mut self) -> Vec<Token<'this>> {
        self.get_tokens().collect()
    }

    /// Gets a tokens iterator streaming over the input.
    fn get_tokens(&mut self) -> Box<dyn Iterator<Item = Token<'this>> + '_> {
        Box::new(std::iter::from_fn(move || self.next_token()))
    }

    /// Calculates the line (1-indexed) of the lexeme from its position in the source text.
    fn get_column(&self, data: &str, lexpos: usize) -> usize
    where
        Self: Sized,
    {
        let begin_line = data[0..lexpos].rfind('\n').map(|v| v + 1).unwrap_or(0);
        (lexpos - begin_line) + 1
    }
}
