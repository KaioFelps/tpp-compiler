use super::Lexer;
use crate::errors::TppCompilerError;
use crate::token::{Token, TokenKind};
use const_format::formatcp;
use itertools::{Itertools, MultiPeek};
use log::debug;
use regex::Regex;
use std::{collections::HashMap, str::Chars, sync::LazyLock};

const NUMBER_PATTERN: &str = r"[\+-]?[0-9]+";

static IDENTIFIER_ALPHABET_SUBSET: [char; 66] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z', 'á', 'Á', 'ã', 'Ã', 'à', 'À', 'é', 'É', 'í', 'Í', 'ó', 'Ó', 'õ', 'Õ',
];

static SCIENTIFIC_NOTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(formatcp!("^[\\+-]?[1-9]\\.[0-9]+[Ee](?:{NUMBER_PATTERN})?"))
        .expect("REGEX de notação científica mal formado")
});

static FLOATING_POINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(formatcp!("^[0-9]*\\.(?:[0-9]+(?:[Ee]{NUMBER_PATTERN})?)?"))
        .expect("REGEX de número flutuante mal formado")
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
// comments ({* and *}) doesn't imply curly brackets being in this array because
// they're parsed before these arrays are considered; they're manually checked
// through lookahead and are not in `CHAR_PAIRS_SYMBOLS_THAT_INTERSECTS_WITH_SINGLE_CHAR_SYMBOLS`
// or whatsoever, hence it doesn't need to be here.
const CHARS_THAT_CAN_BE_SINGLE_OR_PAIRS: [char; 3] = ['<', '>', ':'];
static SINGLE_CHAR_SYMBOLS: LazyLock<HashMap<char, TokenKind>> = LazyLock::new(|| {
    HashMap::from_iter([
        ('(', TokenKind::AbreParentese),
        (')', TokenKind::FechaParentese),
        ('[', TokenKind::AbreColchete),
        (']', TokenKind::FechaColchete),
        // instead, we put them here, since these will be checked **after** comments have been checked
        ('{', TokenKind::AbreChave),
        ('}', TokenKind::FechaChave),
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

enum WasParsingComment {
    Yes,
    No,
}

pub struct ManualDFALexer<'this> {
    /// A view of the whole source code string.
    pub(super) data: &'this str,
    /// The position of the character being analyzed in the source code string.
    pub(super) cursor: usize,
    /// The position the cursor must be moved to on the next char read.
    next_cursor_value: usize,
    /// The current line in the source code.
    pub(super) line_number: usize,
    /// Whether the errors should have `force_key` option enabled.
    force_error_key: bool,
    /// Whether the subsets should consider just a subset of the actual acceptable
    /// subsets. I.e., when `true`, identifiers must be a lexeme over the alphabet
    /// {abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZáÁãÃàÀéÉíÍóÓõÕ}.
    ///
    /// This is useful for the tests that compare this lexer against the
    /// Python implementations.
    use_identifiers_reduced_subset: bool,
    /// A slot to hold a token that was omitted until the next round due to a
    /// implicit [`TokenKind::FimDeSentenca`] emission.
    ///
    /// When `Some(token)`, `token` will be immediately returned on the next
    /// [`Self::next_token`] call.
    pub(super) held_token: Option<Token<'this>>,
    /// Whether [`TokenKind::FimDeSentenca`] tokens should never be emitted.
    pub(super) skip_end_of_statement: bool,
    /// A clone of the previous emitted token. Used to handle
    /// [`TokenKind::FimDeSentenca`] emission.
    pub(super) previous_token: Option<Token<'this>>,
    /// How deep the current token is in nestness (i.e., how many (, {, [ or other wrapper
    /// structures are surrounding the token under the lexer cursor).
    pub(super) cursor_depth: usize,
    pub(super) reached_eof: bool,
}

impl<'this> Lexer<'this> for ManualDFALexer<'this> {
    fn next_token(&mut self) -> Option<Result<Token<'this>, TppCompilerError>> {
        if let Some(token) = self.held_token.take() {
            self.release_token(&token);
            debug!("[Lexer] Dando preferência para o token mantido para a atual tokenização ({token:?}).");
            return Some(Ok(token));
        }

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

            if self.is_valid_identifier_letter(char) || char == '_' {
                let token = self.lookahead_check_identifier_or_reserved_word(&mut iterator, char);
                return self.emit_token(token);
            }

            if char.is_numeric() || char == '+' || char == '-' {
                if let Some(token) = self.lookahead_check_number(&mut iterator, char) {
                    return self.emit_token(token);
                }

                if char.is_numeric() {
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
            }

            self.next_cursor_value += char.len_utf8();

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

            if char == '{' {
                match self.lookahead_check_comment(&mut iterator, char) {
                    Ok(WasParsingComment::Yes) => {
                        continue;
                    }
                    Err(err) => {
                        return Some(Err(err));
                    }
                    _ => (),
                };
            }

            if let Some(token_kind) = SINGLE_CHAR_SYMBOLS.get(&char).cloned() {
                let token = if CHARS_THAT_CAN_BE_SINGLE_OR_PAIRS.contains(&char)
                    && let Some(next_char) = iterator.peek().cloned()
                    && let Some(token_kind) = CHAR_PAIRS_SYMBOLS_THAT_INTERSECTS_WITH_SINGLE_CHAR_SYMBOLS
                        .get(format!("{char}{next_char}").as_str())
                        .cloned()
                {
                    let _ = iterator.next();
                    self.next_cursor_value += next_char.len_utf8();

                    let lexeme = format!("{char}{next_char}");
                    log_lexeme_found(&lexeme, self.cursor, token_kind.clone());
                    self.prepare_token(token_kind, &self.data[self.cursor..self.next_cursor_value])
                } else {
                    log_lexeme_char_found(char, self.cursor, token_kind.clone());
                    self.prepare_token(token_kind, &self.data[self.cursor..self.next_cursor_value])
                };

                return self.emit_token(token);
            }

            if FIRST_CHARS_OF_CHAR_PAIR_TOKENS.contains(&char)
                && let Some(next_char) = iterator.peek().cloned()
                && let Some(token_kind) = CHAR_PAIRS_SYMBOLS.get(format!("{char}{next_char}").as_str()).cloned()
            {
                self.next_cursor_value += next_char.len_utf8();
                let _ = iterator.next();

                let lexeme = format!("{char}{next_char}");
                log_lexeme_found(&lexeme, self.cursor, token_kind.clone());
                let token = self.prepare_token(token_kind, &self.data[self.cursor..self.next_cursor_value]);
                return self.emit_token(token);
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

        self.maybe_generate_eof_token().map(Ok)
    }
}

impl<'this> ManualDFALexer<'this> {
    pub fn new(
        data: &'this str,
        force_error_key: bool,
        use_identifiers_reduced_subset: bool,
        skip_end_of_statement: bool,
    ) -> Self {
        Self {
            cursor: 0,
            next_cursor_value: 0,
            line_number: 1,
            data,
            force_error_key,
            use_identifiers_reduced_subset,
            cursor_depth: 0,
            held_token: None,
            previous_token: None,
            skip_end_of_statement,
            reached_eof: false,
        }
    }

    /// A simple helper for running the token emission routine.
    fn emit_token(&mut self, token: Token<'this>) -> Option<Result<Token<'this>, TppCompilerError>> {
        let token = self.maybe_insert_eos_token(token);
        self.release_token(&token);
        debug!("[Lexer] Armazenado token ({token:?}) como o token anterior da próxima tokenização.");
        Some(Ok(token))
    }

    pub(super) fn prepare_token(&self, kind: TokenKind, value: &'this str) -> Token<'this> {
        Token::new(
            kind,
            value,
            self.cursor,
            self.line_number,
            self.get_column(self.data, self.cursor),
        )
    }

    fn is_valid_identifier_letter(&self, char: char) -> bool {
        self.use_identifiers_reduced_subset && IDENTIFIER_ALPHABET_SUBSET.contains(&char)
            || !self.use_identifiers_reduced_subset && char.is_alphabetic()
    }

    /// Mounts a lexeme and transform it in the ID token (or the most appropriate kind).
    ///
    /// It **does** update the cursor and leave it ready to lex the next char.
    fn lookahead_check_identifier_or_reserved_word(
        &mut self,
        iterator: &mut MultiPeek<Chars<'_>>,
        first_char: char,
    ) -> Token<'this> {
        let mut byte_offset = first_char.len_utf8();
        let mut chars_to_consume = 0;

        iterator.reset_peek();
        while let Some(char) = iterator.peek().cloned() {
            if self.is_valid_identifier_letter(char) || char.is_numeric() || char == '_' {
                byte_offset += char.len_utf8();
                chars_to_consume += 1;
                continue;
            }
            break;
        }

        self.next_cursor_value = self.cursor + byte_offset;
        for _ in 0..chars_to_consume {
            iterator.next();
        }

        let lexeme = &self.data[self.cursor..self.next_cursor_value];

        debug!(
            r#"[Lexer] Formou o lexema "{lexeme}" no intervalo [{}, {}[ do código fonte."#,
            self.cursor,
            self.cursor + byte_offset
        );

        let token_kind = RESERVED_WORDS.get(lexeme).cloned().unwrap_or(TokenKind::Ident);
        log_lexeme_found(lexeme, self.cursor, token_kind.clone());

        self.prepare_token(token_kind, lexeme)
    }

    fn lookahead_check_number(
        &mut self,
        iterator: &mut MultiPeek<Chars<'_>>,
        first_char: char,
    ) -> Option<Token<'this>> {
        let regexes = [&SCIENTIFIC_NOTATION, &FLOATING_POINT];
        let log_kind = ["número por notação científica", "decimal"];
        let kind = [TokenKind::NumNotacaoCientifica, TokenKind::NumPontoFlutuante];

        let view = &self.data[self.cursor..];

        for i in 0..2 {
            if let Some(lexeme_matched) = regexes[i].find(view) {
                let lexeme_str = lexeme_matched.as_str();
                let byte_len = lexeme_str.len();

                self.next_cursor_value = self.cursor + byte_len;

                // since the first charactere has been consumed on the main loop, we do -1
                let chars_to_consume = lexeme_str.chars().count().saturating_sub(1);
                for _ in 0..chars_to_consume {
                    iterator.next();
                }

                debug!(
                    r#"[Lexer] Formou {} "{lexeme_str}" no intervalo [{}, {}[ do código fonte."#,
                    log_kind[i],
                    self.cursor,
                    self.cursor + lexeme_str.len()
                );

                let token = self.prepare_token(kind[i].clone(), lexeme_str);
                return Some(token);
            }
        }

        if !first_char.is_numeric() {
            return None;
        }

        let mut bytes_offset = first_char.len_utf8();
        let mut chars_to_consume = 0;

        iterator.reset_peek();
        while let Some(char) = iterator.peek() {
            if !char.is_numeric() {
                break;
            }
            bytes_offset += char.len_utf8();
            chars_to_consume += 1;
        }

        self.next_cursor_value = self.cursor + bytes_offset;
        for _ in 0..chars_to_consume {
            let _ = iterator.next();
        }

        let lexeme_str = &self.data[self.cursor..(self.cursor + bytes_offset)];

        debug!(
            r#"[Lexer] Formou número inteiro "{lexeme_str}" no intervalo [{}, {}[ do código fonte."#,
            self.cursor,
            self.cursor + bytes_offset
        );

        Some(self.prepare_token(TokenKind::NumInteiro, lexeme_str))
    }

    fn lookahead_check_comment(
        &mut self,
        iterator: &mut MultiPeek<Chars<'_>>,
        first_char: char,
    ) -> Result<WasParsingComment, TppCompilerError> {
        let mut byte_offset = first_char.len_utf8();
        let mut is_within_comment = false;
        let mut ready_to_close_comment = false;

        for char in iterator.by_ref() {
            byte_offset += char.len_utf8();

            if char == '*' {
                if !is_within_comment {
                    is_within_comment = true;
                } else {
                    ready_to_close_comment = true;
                }
                continue;
            }

            if !is_within_comment {
                return Ok(WasParsingComment::No);
            }

            if char == '}' && ready_to_close_comment {
                self.next_cursor_value = self.cursor + byte_offset;
                return Ok(WasParsingComment::Yes);
            } else if ready_to_close_comment {
                // handles cases like `{* fooo * a}`; * was not forming *} at all
                ready_to_close_comment = false
            }

            if char == '\n' {
                self.line_number += 1;
            }
        }

        // if it was indeed an unclosed comment, we gotta advance the cursor and consume the chars up to
        // EOL. this stops the lexer from tokenizing further chars as other tokens, what is unexpected behavior
        if is_within_comment {
            self.next_cursor_value = self.cursor + byte_offset;
            // returning this outside of this if-branch causes '{' as last char from file being parsed as
            // comment instead of ABRE_CHAVE
            return Err(TppCompilerError::new_lexer(
                "ERR-LEX-UNC-COMMENT",
                self.force_error_key,
                "".to_string(),
                self.line_number,
                self.get_column(self.data, self.cursor),
                None,
            ));
        }

        Ok(WasParsingComment::No)
    }
}
