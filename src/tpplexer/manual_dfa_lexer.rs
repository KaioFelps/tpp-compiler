use std::{collections::HashMap, sync::LazyLock};

use const_format::formatcp;
use log::debug;
use regex::Regex;

use crate::{
    token::{Token, TokenKind},
    tpplexer::Lexer,
};

const LETTER: &str = "[a-zA-ZÀ-ÿ]";
const NUMBER: &str = "[\\+-]?[0-9]+";

static IDENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(formatcp!("{LETTER}(?:{LETTER}|_|[0-9])*")).expect("REGEX de identificadores mal formado")
});
static NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(NUMBER).expect("REGEX de número mal formado"));
static SCIENTIFIC_NOTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(formatcp!("{NUMBER}\\.[0-9]+([Ee]{NUMBER})?")).expect("REGEX de número flutuante mal formado")
});

const OTHER_CHARS_TO_IGNORE: [char; 3] = [' ', '\t', '\r'];

static RESERVED_WORDS: LazyLock<HashMap<&str, TokenKind>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("se", TokenKind::Se),
        ("então", TokenKind::Entao),
        ("senão", TokenKind::Senao),
        ("fim", TokenKind::Fim),
        ("repita", TokenKind::Repita),
        ("flutuante", TokenKind::Flutuante),
        ("retorna", TokenKind::Retorna),
        ("até", TokenKind::Ate),
        ("leia", TokenKind::Leia),
        ("escreva", TokenKind::Escreva),
        ("inteiro", TokenKind::Inteiro),
    ])
});

pub struct ManualDFALexer<'this> {
    /// A view of the whole source code string.
    data: &'this str,
    /// The position of the character being analyzed in the source code string.
    cursor: usize,
    /// The position the cursor must be moved to on the next char read.
    next_cursor_value: usize,
    /// The current line in the source code.
    line_number: usize,
}

impl<'this> Lexer<'this> for ManualDFALexer<'this> {
    fn next_token(&mut self) -> Option<Token<'this>> {
        while let Some(char) = self.next_char() {
            if char == '\n' {
                self.line_number += 1;
                continue;
            }

            if OTHER_CHARS_TO_IGNORE.contains(&char) {
                continue;
            }

            if char.is_alphabetic() || char == '_' {
                return Some(self.lookahead_check_identifier_or_reserved_word());
            }

            if char.is_numeric() {
                return self
                    .lookahead_check_scientific_notation()
                    .or(self.lookahead_check_floating())
                    .or(self.lookahead_check_number());
            }
        }

        None
    }
}

impl<'this> ManualDFALexer<'this> {
    pub fn new(data: &'this str) -> Self {
        Self {
            cursor: 0,
            next_cursor_value: 0,
            line_number: 0,
            data,
        }
    }

    /// Look the char `offset` positions ahead from the cursor.
    fn peek_char(&self, offset: usize) -> Option<char> {
        self.data.chars().nth(self.cursor + offset)
    }

    /// Gets the next char and moves the lexer cursor.
    fn next_char(&mut self) -> Option<char> {
        // both start as 0, so the first char won't be skipped on the first call.
        // also this allow the methods to perform easy calculus with the current cursor position.
        self.cursor = self.next_cursor_value;
        let char = self.data.chars().nth(self.cursor);
        if char.is_some() {
            self.next_cursor_value += 1;
        }
        char
    }

    /// Mounts a lexeme and transform it in the ID token (or the most appropriate kind).
    ///
    /// It **does** update the cursor and leave it ready to lex the next char.
    fn lookahead_check_identifier_or_reserved_word(&mut self) -> Token<'this> {
        let mut offset = 1;
        while let Some(char) = self.peek_char(offset) {
            if char.is_alphanumeric() || char == '_' {
                offset += 1;
            }
            break;
        }

        let lexeme = &self.data[self.cursor..offset];

        debug!(
            "[Lexer] Formou o lexema {lexeme} no intervalo [{}, {}[ do código fonte.",
            self.cursor, offset
        );

        let token_kind = RESERVED_WORDS.get(lexeme).cloned().unwrap_or(TokenKind::Ident);
        debug!("[Lexer] Lexema {lexeme} identificado como {token_kind:?}.");
        let token = Token::<'this>::new(
            token_kind,
            lexeme,
            self.line_number,
            self.get_column(&self.data, self.cursor),
        );

        // next cursor is already always 1 position ahead of cursor
        self.next_cursor_value += offset - 1;

        token
    }

    fn lookahead_check_scientific_notation(&mut self) -> Option<Token<'this>> {
        // we use this buffer to match a REGEX over a single `char` without having
        // to allocate a whole String just to call `Regex::is_match`.
        let mut char_buffer = [0; 4];
        let char_as_str = self
            .data
            .chars()
            .nth(self.cursor)
            .unwrap()
            .encode_utf8(&mut char_buffer);

        if let Some(lexeme) = SCIENTIFIC_NOTATION.find_at(self.data, self.cursor) {
            let chars_to_skip = lexeme.as_str().chars().count();
            // -1 due to 0-indexing and another -1 due to next_cursor_value being already 1 position ahead
            self.next_cursor_value += chars_to_skip - 2;
        }

        None
    }

    fn lookahead_check_floating(&mut self) -> Option<Token<'this>> {
        None
    }

    fn lookahead_check_number(&mut self) -> Option<Token<'this>> {
        None
    }
}
