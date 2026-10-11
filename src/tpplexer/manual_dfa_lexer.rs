use crate::{
    errors::TppCompilerError,
    token::{Token, TokenKind},
    tpplexer::Lexer,
};
use const_format::formatcp;
use itertools::{Itertools, MultiPeek};
use log::debug;
use regex::Regex;
use std::{collections::HashMap, str::Chars, sync::LazyLock};

const NUMBER: &str = "[\\+-]?[0-9]+";

static NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(NUMBER).expect("REGEX de número mal formado"));
static SCIENTIFIC_NOTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(formatcp!("[-\\+]?[1-9]\\.[0-9]+[Ee](?:{NUMBER})?")).expect("REGEX de notação científica mal formado")
});
static FLOATING_POINT: LazyLock<Regex> = LazyLock::new(|| {
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

/// Chars that can be either a single symbol but can also form a pair
const CHARS_THAT_CAN_BE_SINGLE_OR_PAIRS: [char; 3] = ['<', '>', ':'];
static SINGLE_CHAR_SYMBOLS: LazyLock<HashMap<char, TokenKind>> = LazyLock::new(|| {
    HashMap::from_iter([
        ('(', TokenKind::AbreParentese),
        (')', TokenKind::FechaParentese),
        ('[', TokenKind::AbreColchete),
        (']', TokenKind::FechaColchete),
        ('!', TokenKind::Nao),
        ('=', TokenKind::Igual),
        ('<', TokenKind::Menor),
        ('>', TokenKind::Maior),
        (',', TokenKind::Virgula),
        ('/', TokenKind::Divide),
        ('*', TokenKind::Vezes),
        ('+', TokenKind::Mais),
        ('-', TokenKind::Menos),
        (':', TokenKind::DoisPontos),
    ])
});
/// Pairs of chars that forms a token but that intersects with single char symbols.
static CHAR_PAIRS_SYMBOLS_THAT_INTERSECTS_WITH_SINGLE_CHAR_SYMBOLS: LazyLock<HashMap<&str, TokenKind>> =
    LazyLock::new(|| {
        HashMap::from_iter([
            ("<>", TokenKind::Diferente),
            ("<=", TokenKind::MenorIgual),
            (">=", TokenKind::MaiorIgual),
            (":=", TokenKind::Atribuicao),
        ])
    });

/// Chars that init a pair of chars that forms a valid token. None of these
/// intersects with tokens formed by single chars. (I.e., it's disjoint of single-chair tokens set.)
const FIRST_CHARS_OF_CHAR_PAIR_TOKENS: [char; 3] = ['&', '|', ':'];
static CHAR_PAIRS_SYMBOLS: LazyLock<HashMap<&str, TokenKind>> =
    LazyLock::new(|| HashMap::from_iter([("&&", TokenKind::E), ("||", TokenKind::Ou)]));

#[inline]
fn log_lexeme_found(lexeme: &str, pos: usize, token_kind: TokenKind) {
    debug!(r#"[Lexer] Lexema "{lexeme}" identificado na posição {pos} como {token_kind:?}."#);
}

#[inline]
fn log_lexeme_char_found(lexeme: char, pos: usize, token_kind: TokenKind) {
    debug!(r#"[Lexer] Lexema "{lexeme}" identificado na posição {pos} como {token_kind:?}."#);
}

pub struct ManualDFALexer<'this> {
    /// A view of the whole source code string.
    data: &'this str,
    /// The position of the character being analyzed in the source code string.
    cursor: usize,
    /// The position the cursor must be moved to on the next char read.
    next_cursor_value: usize,
    /// The current line in the source code.
    line_number: usize,
    /// Whether the errors should have `force_key` option enabled.
    force_error_key: bool,
}

impl<'this> Lexer<'this> for ManualDFALexer<'this> {
    fn next_token(&mut self) -> Option<Result<Token<'this>, TppCompilerError>> {
        // last calls to `next_token` might have updated `self.next_cursor_value` but left
        // without updating self.cursor, so we defer this responsability to `next_token` itself.
        // won't affect the first char reading since `next_cursor_value` is initialized as the
        // same value of `self.cursor`.
        self.cursor = self.next_cursor_value;
        debug!(
            "[Lexer] Iniciando tokenização partindo do índice {}.",
            self.next_cursor_value
        );

        let view = &self.data[self.cursor..];
        let mut iterator = view.chars().multipeek();
        while let Some(char) = iterator.next() {
            // we need to keep track of changes in `self.next_cursor_value` on the begin
            // of each iteration so that tokens can track lexemes correctly
            self.cursor = self.next_cursor_value;
            debug!("[Lexer] Aplicando análise léxica em '{char}' ({}).", self.cursor);

            if char.is_alphabetic() || char == '_' {
                return Some(Ok(self.lookahead_check_identifier_or_reserved_word(&mut iterator)));
            }

            if char.is_numeric() || char == '+' || char == '-' {
                if let Some(token) = self.lookahead_check_number(&mut iterator) {
                    return Some(Ok(token));
                }

                let err = TppCompilerError::new_lexer(
                    "ERR-LEX-INV-CHAR",
                    self.force_error_key,
                    char.to_string(),
                    self.line_number,
                    self.get_column(self.data, self.cursor),
                    None,
                );

                return Some(Err(err));
            }

            self.next_cursor_value += 1;

            if char == '\n' {
                self.line_number += 1;
                debug!(
                    "[Lexer] Quebra de linha identificada. Mudando para a linha {}.",
                    self.line_number
                );
                continue;
            }

            if OTHER_CHARS_TO_IGNORE.contains(&char) {
                continue;
            }

            if let Some(token_kind) = SINGLE_CHAR_SYMBOLS.get(&char).cloned() {
                if CHARS_THAT_CAN_BE_SINGLE_OR_PAIRS.contains(&char)
                    && let Some(next_char) = iterator.peek().cloned()
                    && let Some(token_kind) = CHAR_PAIRS_SYMBOLS_THAT_INTERSECTS_WITH_SINGLE_CHAR_SYMBOLS
                        .get(format!("{char}{next_char}").as_str())
                        .cloned()
                {
                    let _ = iterator.next();
                    self.next_cursor_value += 1;

                    let lexeme = format!("{char}{next_char}");
                    log_lexeme_found(&lexeme, self.cursor, token_kind.clone());
                    let token = self.prepare_token(token_kind, &self.data[self.cursor..self.next_cursor_value]);
                    return Some(Ok(token));
                }

                log_lexeme_char_found(char, self.cursor, token_kind.clone());
                let token = self.prepare_token(TokenKind::Igual, &self.data[self.cursor..self.next_cursor_value]);
                return Some(Ok(token));
            }

            if FIRST_CHARS_OF_CHAR_PAIR_TOKENS.contains(&char)
                && let Some(next_char) = iterator.peek().cloned()
                && let Some(token_kind) = CHAR_PAIRS_SYMBOLS.get(format!("{char}{next_char}").as_str()).cloned()
            {
                self.next_cursor_value += 1;
                let _ = iterator.next();

                let lexeme = format!("{char}{next_char}");
                log_lexeme_found(&lexeme, self.cursor, token_kind.clone());
                let token = self.prepare_token(token_kind, &self.data[self.cursor..self.next_cursor_value]);
                return Some(Ok(token));
            }

            if char == '{' {
                if let Err(err) = self.lookahead_check_comment(&mut iterator) {
                    return Some(Err(err));
                }
                continue;
            }

            return Some(Err(TppCompilerError::new_lexer(
                "ERR-LEX-INV-CHAR",
                self.force_error_key,
                char.to_string(),
                self.line_number,
                self.get_column(self.data, self.cursor),
                None,
            )));
        }

        None
    }
}

impl<'this> ManualDFALexer<'this> {
    pub fn new(data: &'this str, force_error_key: bool) -> Self {
        Self {
            cursor: 0,
            next_cursor_value: 0,
            line_number: 1,
            data,
            force_error_key,
        }
    }

    fn prepare_token(&self, kind: TokenKind, value: &'this str) -> Token<'this> {
        Token::new(
            kind,
            value,
            self.cursor,
            self.line_number,
            self.get_column(self.data, self.cursor),
        )
    }

    /// Mounts a lexeme and transform it in the ID token (or the most appropriate kind).
    ///
    /// It **does** update the cursor and leave it ready to lex the next char.
    fn lookahead_check_identifier_or_reserved_word(&mut self, iterator: &mut MultiPeek<Chars<'_>>) -> Token<'this> {
        let mut offset = 1;
        iterator.reset_peek();
        while let Some(char) = iterator.peek().cloned() {
            if char.is_alphanumeric() || char == '_' {
                offset += 1;
                continue;
            }
            break;
        }

        let lexeme = &self.data[self.cursor..(self.cursor + offset)];

        debug!(
            r#"[Lexer] Formou o lexema "{lexeme}" no intervalo [{}, {}[ do código fonte."#,
            self.cursor,
            self.cursor + offset
        );

        self.next_cursor_value = self.cursor + offset;
        let _ = iterator.nth(offset);

        let token_kind = RESERVED_WORDS.get(lexeme).cloned().unwrap_or(TokenKind::Ident);
        log_lexeme_found(lexeme, self.cursor, token_kind.clone());
        self.prepare_token(token_kind, lexeme)
    }

    fn lookahead_check_number(&mut self, iterator: &mut MultiPeek<Chars<'_>>) -> Option<Token<'this>> {
        let regexes = [&SCIENTIFIC_NOTATION, &FLOATING_POINT, &NUMBER_REGEX];
        let log_kind = ["número por notação científica", "decimal", "número inteiro"];
        let kind = [
            TokenKind::NumNotacaoCientifica,
            TokenKind::NumPontoFlutuante,
            TokenKind::NumInteiro,
        ];

        for i in 0..3 {
            let regex = regexes[i];

            if let Some(lexeme) = regex.find_at(self.data, self.cursor) {
                let lexeme_str = lexeme.as_str();
                // creates a temporary iterator, which might seem heavy, but it has just few characters of length so
                // it weights nothing at all
                let lexeme_len = lexeme_str.chars().count();

                self.next_cursor_value = self.cursor + lexeme_len;
                let _ = iterator.nth(lexeme_len);

                debug!(
                    r#"[Lexer] Formou {} "{lexeme_str}" no intervalo [{}, {}[ do código fonte."#,
                    log_kind[i],
                    self.cursor,
                    self.cursor + lexeme_len
                );

                let token = self.prepare_token(kind[i].clone(), lexeme_str);
                return Some(token);
            }
        }

        None
    }

    fn lookahead_check_comment(&mut self, iterator: &mut MultiPeek<Chars<'_>>) -> Result<(), TppCompilerError> {
        let mut offset = 1;

        for char in iterator.by_ref() {
            offset += 1;

            if char == '\n' {
                self.line_number += 1;
            }

            if char == '}' {
                self.next_cursor_value = self.cursor + offset;
                return Ok(());
            }
        }

        Err(TppCompilerError::new_lexer(
            "ERR-LEX-UNC-COMMENT",
            self.force_error_key,
            self.data[self.cursor..(self.cursor + offset)].to_string(),
            self.line_number,
            self.get_column(self.data, self.cursor),
            None,
        ))
    }
}
