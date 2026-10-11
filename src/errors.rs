use configparser::ini::Ini;
use log::warn;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ErrorSection {
    Global,
    Lexer,
    Parser,
    // SemanticError,
    // CodeGenError,
}

impl<T: AsRef<str>> From<T> for ErrorSection {
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

impl Display for ErrorSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ErrorSection::Global => "GlobalError".to_string(),
            ErrorSection::Lexer => "LexerError".to_string(),
            ErrorSection::Parser => "ParserError".to_string(),
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
static ERROR_MESSAGES: LazyLock<HashMap<ErrorSection, HashMap<&'static str, &'static str>>> = LazyLock::new(|| {
    ERROR_MESSAGES_DESCRIPTORS
        .into_iter()
        .map(|config| Ini::new().read(config.to_string()))
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
                    (ErrorSection::from(section), errors)
                })
                // map[section]=errors map
                .collect::<HashMap<_, _>>()
        })
        // flatmap kept this as a map[section]=errors map
        .collect::<HashMap<_, _>>()
});

#[derive(Debug)]
pub struct TppCompilerError {
    section: ErrorSection,
    key: String,
    force_key: bool,
}

impl TppCompilerError {
    pub fn new<T: ToString>(section: ErrorSection, key: T) -> Self {
        Self {
            key: key.to_string().to_lowercase(),
            section,
            force_key: false,
        }
    }

    pub fn new_with_force_key<T: ToString>(section: ErrorSection, key: T, force_key: bool) -> Self {
        Self {
            key: key.to_string().to_lowercase(),
            section,
            force_key,
        }
    }
}

impl Display for TppCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.force_key {
            return write!(f, "{}", self.key);
        }

        let content = if let Some(message) = ERROR_MESSAGES
            .get(&self.section)
            .and_then(|section| section.get(self.key.as_str()))
        {
            *message
        } else {
            &self.key
        };

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
