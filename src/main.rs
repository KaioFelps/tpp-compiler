mod cli;
mod errors;

use std::fs::OpenOptions;

use crate::cli::Args;
use clap::Parser;
use log::debug;

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging()?;
    let args = Args::parse();
    debug!("Leitura de argumentos finalizada.");

    println!(
        "Hello, world! usou key: {}; arquivos fontes: {:#?}",
        args.use_errors_key, args.source_files
    );

    Ok(())
}
