use configparser::ini::Ini;
use core::fmt;
use itertools::Itertools;
use log::warn;
use std::{collections::HashMap, fmt::Display, ops::Deref, sync::LazyLock};

use crate::token::{OwnedToken, TokenKind};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ErrorMessagesSection {
    Global,
    Lexer,
    Parser,
    // SemanticError,
    // CodeGenError,
}

impl<T: AsRef<str>> From<T> for ErrorMessagesSection {
    fn from(value: T) -> Self {
        let value = value.as_ref();
        match value {
            "GlobalErrors" | "globalerrors" => Self::Global,
            "LexerErrors" | "lexererrors" => Self::Lexer,
            "ParserErrors" | "parsererrors" => Self::Parser,
            // "semanticerror" => Self::SemanticError,
            // "codegenerror" => Self::CodeGenError,
            _ => panic!("Um erro não registrado tentou ser convertido em uma variante de `ErrorSection`: {value}"),
        }
    }
}

impl Display for ErrorMessagesSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ErrorMessagesSection::Global => "GlobalError".to_string(),
            ErrorMessagesSection::Lexer => "LexerError".to_string(),
            ErrorMessagesSection::Parser => "ParserError".to_string(),
            // ErrorSection::SemanticError => "SemanticError".to_string(),
            // ErrorSection::CodeGenError => "CodeGenError".to_string(),
        };

        write!(f, "{string}")
    }
}

/// The _ini_ config for each error kind
const ERROR_MESSAGES_DESCRIPTORS: [&str; 3] = [
    include_str!("errors/GlobalErrorMessages.properties"),
    include_str!("errors/LexerErrorMessages.properties"),
    include_str!("errors/ParserErrorMessages.properties"),
    // include_str!("tppsema/SemaErrorMessages.properties"),
    // include_str!("tppcodegen/CodeGenErrorMessages.propertie"),
];

/// The sections of errors and their key-value pairs in-memory.
static ERROR_MESSAGES: LazyLock<HashMap<ErrorMessagesSection, HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| {
        ERROR_MESSAGES_DESCRIPTORS
            .into_iter()
            .map(|config| {
                let mut ini = Ini::new_cs();
                ini.set_multiline(true);
                ini.read(config.to_string())
            })
            .filter_map(|res| match res {
                Ok(res) => Some(res),
                Err(err) => {
                    warn!("Não foi possível interpretar o arquivo de configuração de erros: {err}.",);
                    None
                }
            })
            .flat_map(|sections| {
                sections
                    .into_iter()
                    .map(|(section, errors_map)| {
                        let errors = errors_map
                            .into_iter()
                            .map(|(key, message)| (key.leak() as &str, message.map(|msg| msg.leak() as &str)))
                            .filter_map(|(key, message)| message.map(|message| (key, message)))
                            .collect::<HashMap<_, _>>();
                        (ErrorMessagesSection::from(section), errors)
                    })
                    // map[section]=errors map
                    .collect::<HashMap<_, _>>()
            })
            // flatmap kept this as a map[section]=errors map
            .collect::<HashMap<_, _>>()
    });

#[derive(Debug)]
pub enum ErrorKind {
    Global {
        file: Option<String>,
    },
    Lexer {
        file: Option<String>,
        line: usize,
        column: usize,
        lexeme: String,
    },
    Parser {
        file: String,
        /// The token that was currently being analysed when the error occurred.
        ///
        /// ## Usage
        /// If present, it can be used in error messages through the following placeholders:
        /// - `{tok}` is replaced with `OwnedToken` simple debug print (`"{:?}"`)
        /// - `{tok:kind}` is replaced with `OwnedToken::kind`
        /// - `{tok:value}` is replaced with `OwnedToken::value`
        /// - `{tok:line}` is replaced with `OwnedToken::line_number`
        /// - `{tok:col}` is replaced with `OwnedToken::col_number`
        current_token: Option<OwnedToken>,
        /// If an extra token other than the current one being analysed can provide
        /// better context for the error message (e.g., the next token that failed upon peek
        /// and caused the error), it can be placed here and will replace `{extra_tok}`
        /// placeholders upon error messages.
        ///
        /// ## Usage
        /// - `{extra_tok}` is replaced with `OwnedToken` simple debug print (`"{:?}"`)
        /// - `{extra_tok:kind}` is replaced with `OwnedToken::kind`
        /// - `{extra_tok:value}` is replaced with `OwnedToken::value`
        /// - `{extra_tok:line}` is replaced with `OwnedToken::line_number`
        /// - `{extra_tok:col}` is replaced with `OwnedToken::col_number`
        extra_token: Option<OwnedToken>,
    },
}

// moved stuff into inner to the heap because rust compiler was warning this error was too large (~128 bytes)
#[derive(Debug)]
struct TppCompilerErrorInner {
    kind: ErrorKind,
    key: String,
    force_key: bool,
}

#[derive(Debug)]
pub struct TppCompilerError(Box<TppCompilerErrorInner>);

impl TppCompilerError {
    pub fn new_global<T: ToString>(key: T, force_key: bool, file: Option<String>) -> Self {
        Self(Box::new(TppCompilerErrorInner {
            key: key.to_string(),
            force_key,
            kind: ErrorKind::Global { file },
        }))
    }

    pub fn new_lexer<T: ToString>(
        key: T,
        force_key: bool,
        lexeme: String,
        line: usize,
        column: usize,
        file: Option<String>,
    ) -> Self {
        Self(Box::new(TppCompilerErrorInner {
            force_key,
            kind: ErrorKind::Lexer {
                file,
                line,
                column,
                lexeme,
            },
            key: key.to_string(),
        }))
    }

    pub fn new_parser<T: ToString>(
        key: T,
        force_key: bool,
        file: String,
        current_token: Option<OwnedToken>,
        extra_token: Option<OwnedToken>,
    ) -> Self {
        Self(Box::new(TppCompilerErrorInner {
            key: key.to_string(),
            force_key,
            kind: ErrorKind::Parser {
                file,
                current_token,
                extra_token,
            },
        }))
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.set_file(file);
        self
    }

    pub fn set_file(&mut self, new_file: String) {
        match &mut self.0.kind {
            ErrorKind::Global { file, .. } | ErrorKind::Lexer { file, .. } => {
                *file = Some(new_file);
            }
            ErrorKind::Parser { file, .. } => {
                *file = new_file;
            }
        };
    }
}

impl Display for TppCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.force_key {
            return write!(f, "{}", self.0.key);
        }

        let section = match &self.0.kind {
            ErrorKind::Global { .. } => ErrorMessagesSection::Global,
            ErrorKind::Lexer { .. } => ErrorMessagesSection::Lexer,
            ErrorKind::Parser { .. } => ErrorMessagesSection::Parser,
        };

        let mut content = if let Some(message) = ERROR_MESSAGES
            .get(&section)
            .and_then(|section| section.get(self.0.key.as_str()))
        {
            message.to_string()
        } else {
            self.0.key.clone()
        };

        match &self.0.kind {
            ErrorKind::Global { file } => content = content.replace("{file}", file.as_deref().unwrap_or_default()),
            ErrorKind::Parser {
                file,
                extra_token,
                current_token,
            } => {
                content = content.replace("{file}", file.deref());

                if let Some(token) = current_token {
                    content = content
                        .replace("{tok}", &format!("{token:?}"))
                        .replace("{tok:kind}", &token.kind.to_string())
                        .replace("{tok:value}", pretty_end_of_statement_value(token))
                        .replace("{tok:line}", &token.line_number.to_string())
                        .replace("{tok:col}", &token.col_number.to_string());
                }

                if let Some(token) = extra_token {
                    content = content
                        .replace("{extra_tok}", &format!("{token:?}"))
                        .replace("{extra_tok:kind}", &token.kind.to_string())
                        .replace("{extra_tok:value}", pretty_end_of_statement_value(token))
                        .replace("{extra_tok:line}", &token.line_number.to_string())
                        .replace("{extra_tok:col}", &token.col_number.to_string());
                }
            }
            ErrorKind::Lexer {
                file,
                line,
                column,
                lexeme,
            } => {
                content = content
                    .replace("{file}", file.as_deref().unwrap_or_default())
                    .replace("{line}", &line.to_string())
                    .replace("{column}", &column.to_string())
                    .replace("{lexeme}", lexeme);
            }
        };

        write!(f, "{content}")
    }
}

fn pretty_end_of_statement_value(token: &OwnedToken) -> &str {
    if token.kind == TokenKind::FimDeSentenca {
        return "<MARCADOR DE FIM DE SENTENÇA>";
    }

    token.value.as_str()
}

impl std::error::Error for TppCompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        &self.0.key
    }
}

#[derive(Debug)]
pub struct TppCompilerErrorsBag(pub Vec<TppCompilerError>);

impl TppCompilerErrorsBag {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn append_error(&mut self, error: TppCompilerError) {
        self.0.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for TppCompilerErrorsBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl IntoIterator for TppCompilerErrorsBag {
    type Item = anyhow::Error;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(anyhow::Error::from).collect_vec().into_iter()
    }
}

impl std::error::Error for TppCompilerErrorsBag {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Debug)]
pub struct AnyErrorsBag(pub Vec<anyhow::Error>);

impl AnyErrorsBag {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn append_error<T: Into<anyhow::Error>>(&mut self, error: T) {
        self.0.push(error.into());
    }

    pub fn with_error<T: Into<anyhow::Error>>(mut self, error: T) -> Self {
        self.0.push(error.into());
        self
    }
}

impl fmt::Display for AnyErrorsBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AnyErrorsBag {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
