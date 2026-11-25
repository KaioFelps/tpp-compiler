use log::debug;

use crate::{
    lexer::{Lexer, ManualDFALexer},
    token::{Token, TokenKind},
};

const TERMINATING_TOKENS: [TokenKind; 8] = [
    TokenKind::Ident,
    TokenKind::NumInteiro,
    TokenKind::NumPontoFlutuante,
    TokenKind::NumNotacaoCientifica,
    TokenKind::FechaParentese,
    TokenKind::FechaColchete,
    TokenKind::FechaChave,
    TokenKind::Fim,
];

const STATEMENT_BOUNDARY_KEYWORDS: [TokenKind; 11] = [
    TokenKind::Se,
    TokenKind::Entao,
    TokenKind::Senao,
    TokenKind::Fim,
    TokenKind::Repita,
    TokenKind::Ate,
    TokenKind::Retorna,
    TokenKind::Leia,
    TokenKind::Escreva,
    TokenKind::Inteiro,
    TokenKind::Flutuante,
];

const OPENING_TOKENS: [TokenKind; 3] = [TokenKind::AbreParentese, TokenKind::AbreColchete, TokenKind::AbreChave];

const CLOSING_TOKENS: [TokenKind; 3] = [
    TokenKind::FechaParentese,
    TokenKind::FechaColchete,
    TokenKind::FechaChave,
];

impl<'this> ManualDFALexer<'this> {
    /// Performs the out-going token routine related to end of statement token management.
    pub(super) fn release_token(&mut self, token: &Token<'this>) {
        self.previous_token = Some(token.clone());

        if OPENING_TOKENS.contains(token.kind()) {
            self.cursor_depth += 1;
        }

        if CLOSING_TOKENS.contains(token.kind()) {
            self.cursor_depth = self.cursor_depth.saturating_sub(1);
        }
    }

    /// If the current token indicates the end of a statement, reserves a
    /// [`TokenKind::FimDeSentenca`] token that must be evaluated at the
    /// next tokenization.
    ///
    /// ## Note
    /// It's up to the callee to consider [`Self::held_token`] on the next
    /// [`Self::next_token`] call.
    pub(super) fn maybe_insert_eos_token(&mut self, current_token: Token<'this>) -> Token<'this> {
        if self.skip_end_of_statement {
            return current_token;
        }

        let has_broken_line = self
            .previous_token
            .as_ref()
            .is_some_and(|previous_token| previous_token.line_number() < current_token.line_number());

        // command/block keywords in newlines cannot be within (), [] nor {}, hence
        // we reset the cursor depth and leave this error to the parser step
        if has_broken_line && STATEMENT_BOUNDARY_KEYWORDS.contains(current_token.kind()) {
            self.cursor_depth = 0;
        }

        let is_not_inside_wrapper_operators = self.cursor_depth == 0;

        let previous_token_was_terminating = self
            .previous_token
            .as_ref()
            .is_some_and(|prev_token| TERMINATING_TOKENS.contains(prev_token.kind()));

        debug!(
            "[Lexer] Para o token ({current_token:?}): quebrou a linha? {has_broken_line}; \
            antecessor é terminador? {previous_token_was_terminating}; está aninhado? {} (nível {}).",
            !is_not_inside_wrapper_operators, self.cursor_depth
        );

        if has_broken_line
            && previous_token_was_terminating
            && is_not_inside_wrapper_operators
            // just a fancy way of getting it without calling `.unwrap`, since we _know_ it is
            // `Some` due to the `previous_token_was_terminating`
            && let Some(prev_token) = &self.previous_token
        {
            // we don't need smth like prev_token.value().chars().map(|c| c.len_utf8()).sum() here cuz
            // str::len() already is the utf8 len, but in O(1) (str keeps track of it's own utf8 bytes size)
            let eos_byte_position = prev_token.value_begin_pos() + prev_token.value().len();
            let end_of_statement_token = Token::new(
                TokenKind::FimDeSentenca,
                ";;",
                eos_byte_position,
                prev_token.line_number(),
                self.get_column(self.data, eos_byte_position),
            );

            debug!("[Lexer] Inserido token de final de sentença: {end_of_statement_token:?}.");
            debug!("[Lexer] Armazenando o token atual ({current_token:?}) para a próxima tokenização.");
            self.held_token = Some(current_token);
            return end_of_statement_token;
        }

        current_token
    }

    /// Generates a [`TokenKind::FimDeSentenca`] token due to end of file
    /// if [`Self::skip_end_of_statement`] is disabled.
    pub(super) fn maybe_generate_eof_token(&mut self) -> Option<Token<'this>> {
        if self.skip_end_of_statement || self.reached_eof || self.cursor_depth != 0 {
            return None;
        }

        let prev = self.previous_token.as_ref()?;
        if !TERMINATING_TOKENS.contains(prev.kind()) {
            return None;
        }

        self.reached_eof = true;
        Some(self.prepare_token(TokenKind::FimDeSentenca, ";;"))
    }
}
