mod cli;
mod errors;
mod token;
mod tpplexer;

use crate::{
    cli::Args,
    errors::TppCompilerError,
    token::Token,
    tpplexer::{Lexer, ManualDFALexer},
};
use clap::Parser;
use log::{debug, info};
use std::{
    collections::HashMap,
    fs::{OpenOptions, exists, read_to_string},
};
use tracing_subscriber::EnvFilter;

fn init_logging() -> anyhow::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("tpp_compiler.log")?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(file)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();

    Ok(())
}

fn validate_source_files(args: &Args) -> Result<(), TppCompilerError> {
    if args.source_files.is_empty() {
        return Err(TppCompilerError::new_global("ERR-GLOB-USE", args.use_errors_key, None));
    }

    if !args.source_files.iter().all(|path| path.ends_with(".tpp")) {
        return Err(TppCompilerError::new_global(
            "ERR-GLOB-NOT-TPP",
            args.use_errors_key,
            None,
        ));
    }

    if args.source_files.iter().any(|path| !exists(path).is_ok_and(|v| v)) {
        return Err(TppCompilerError::new_global(
            "ERR-GLOB-FILE-NOT-EXISTS",
            args.use_errors_key,
            None,
        ));
    }

    Ok(())
}

fn acquire_source_codes(args: &Args) -> anyhow::Result<HashMap<&str, String>> {
    let mut source_codes = HashMap::<&str, String>::new();

    for file in &args.source_files {
        info!("[acquire_source_codes::{file}] Lendo o conteúdo do arquivo de código-fonte");
        let data = read_to_string(file)?;
        debug!("[acquire_source_codes::{file}] Lido o conteúdo do arquivo");
        source_codes.insert(file.as_str(), data);
        debug!("[acquire_source_codes::{file}] Finalizada tokenização do arquivo");
    }

    Ok(source_codes)
}

fn lex_source_codes<'a>(
    args: &Args,
    source_codes: &'a HashMap<&'a str, String>,
) -> anyhow::Result<HashMap<&'a str, Vec<Token<'a>>>> {
    let mut files_tokens = HashMap::<&'a str, Vec<Token<'a>>>::new();

    for (file, data) in source_codes {
        info!("[lex_source_files::{file}] Iniciando a análise léxica do arquivo");
        let mut lexer = ManualDFALexer::new(data.as_str(), args.use_errors_key);
        debug!("[lex_source_files::{file}] Instanciado o lexer para iniciar a análise");
        let tokens = lexer.tokenize().map_err(|err| err.with_file(file.to_string()))?;
        files_tokens.insert(*file, tokens);
        debug!("[lex_source_files::{file}] Finalizada a tokenização do arquivo");
    }

    Ok(files_tokens)
}

fn main() -> anyhow::Result<()> {
    init_logging()?;
    let args = Args::parse();
    debug!("Leitura de argumentos finalizada.");

    validate_source_files(&args)?;

    let source_codes = acquire_source_codes(&args)?;
    let source_files_tokens = lex_source_codes(&args, &source_codes)?;

    if source_files_tokens.len() > 1 {
        for (file, tokens) in source_files_tokens {
            println!("{file}");
            tokens.iter().for_each(|token| println!("\t{token}"));
        }
    } else if source_files_tokens.len() == 1 {
        source_files_tokens.values().collect::<Vec<_>>()[0]
            .iter()
            .for_each(|token| println!("{token}"));
    }

    Ok(())
}
