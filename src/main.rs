mod cli;
mod errors;

use std::fs::{OpenOptions, exists};

use crate::{
    cli::Args,
    errors::{ErrorSection, TppCompilerError},
};
use clap::Parser;
use log::{debug, info};

fn init_logging() -> anyhow::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("tpp_compiler.log")?;

    tracing_subscriber::fmt()
        .with_writer(file)
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();

    Ok(())
}

fn validate_source_files(args: &Args) -> Result<(), TppCompilerError> {
    if args.source_files.is_empty() {
        return Err(TppCompilerError::new_with_force_key(
            ErrorSection::GlobalError,
            "ERR-GLOB-USE",
            args.use_errors_key,
        ));
    }

    if !args.source_files.iter().all(|path| path.ends_with(".tpp")) {
        return Err(TppCompilerError::new_with_force_key(
            ErrorSection::GlobalError,
            "ERR-GLOB-NOT-TPP",
            args.use_errors_key,
        ));
    }

    if args.source_files.iter().any(|path| !exists(path).is_ok_and(|v| v)) {
        return Err(TppCompilerError::new_with_force_key(
            ErrorSection::GlobalError,
            "ERR-GLOB-FILE-NOT-EXISTS",
            args.use_errors_key,
        ));
    }

    Ok(())
}

fn lex_source_files(args: &Args) {
    for file in &args.source_files {
        info!("Iniciando a análise léxica do arquivo \"{file}\".");
    }
}

fn main() -> anyhow::Result<()> {
    init_logging()?;
    let args = Args::parse();
    debug!("Leitura de argumentos finalizada.");

    validate_source_files(&args)?;

    lex_source_files(&args);

    println!(
        "Hello, world! usou key: {}; arquivos fontes: {:#?}",
        args.use_errors_key, args.source_files
    );

    Ok(())
}
