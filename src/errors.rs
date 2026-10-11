use configparser::ini::Ini;
use log::warn;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

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
    include_str!("GlobalErrorMessages.properties"),
    include_str!("tpplexer/LexerErrorMessages.properties"),
    include_str!("tppparser/ParserErrorMessages.properties"),
    // include_str!("tppsema/SemaErrorMessages.properties"),
    // include_str!("tppcodegen/CodeGenErrorMessages.propertie"),
];

/// The sections of errors and their key-value pairs in-memory.
static ERROR_MESSAGES: LazyLock<HashMap<ErrorMessagesSection, HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| {
        ERROR_MESSAGES_DESCRIPTORS
            .into_iter()
            .map(|config| Ini::new_cs().read(config.to_string()))
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
    Parser,
}

#[derive(Debug)]
pub struct TppCompilerError {
    kind: ErrorKind,
    key: String,
    force_key: bool,
}

impl TppCompilerError {
    pub fn new<T: ToString>(kind: ErrorKind, key: T, force_key: bool) -> Self {
        Self {
            kind,
            key: key.to_string(),
            force_key,
        }
    }

    pub fn new_global<T: ToString>(key: T, force_key: bool, file: Option<String>) -> Self {
        Self {
            key: key.to_string(),
            force_key,
            kind: ErrorKind::Global { file },
        }
    }

    pub fn new_lexer<T: ToString>(
        key: T,
        force_key: bool,
        lexeme: String,
        line: usize,
        column: usize,
        file: Option<String>,
    ) -> Self {
        Self {
            force_key,
            kind: ErrorKind::Lexer {
                file,
                line,
                column,
                lexeme,
            },
            key: key.to_string(),
        }
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.set_file(file);
        self
    }

    pub fn set_file(&mut self, file: String) {
        self.kind = match &self.kind {
            ErrorKind::Global { file: _ } => ErrorKind::Global { file: Some(file) },
            ErrorKind::Lexer {
                file: _,
                line,
                column,
                lexeme,
            } => ErrorKind::Lexer {
                file: Some(file),
                line: *line,
                column: *column,
                lexeme: lexeme.clone(),
            },
            ErrorKind::Parser => ErrorKind::Parser,
        };
    }
}

impl Display for TppCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.force_key {
            return write!(f, "{}", self.key);
        }

        let section = match &self.kind {
            ErrorKind::Global { file: _ } => ErrorMessagesSection::Global,
            ErrorKind::Lexer {
                file: _,
                line: _,
                column: _,
                lexeme: _,
            } => ErrorMessagesSection::Lexer,
            ErrorKind::Parser => ErrorMessagesSection::Parser,
        };

        let mut content = if let Some(message) = ERROR_MESSAGES
            .get(&section)
            .and_then(|section| section.get(self.key.as_str()))
        {
            message.to_string()
        } else {
            self.key.clone()
        };

        match &self.kind {
            ErrorKind::Global { file } => {
                content = content.replace("{file}", file.as_deref().unwrap_or_default());
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
            ErrorKind::Parser => {}
        }

        write!(f, "{content}")
    }
}

impl std::error::Error for TppCompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        &self.key
    }
}
