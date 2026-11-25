mod cli;
mod errors;
mod lexer;
mod parser;
mod token;

use crate::{
    cli::{Args, CompilerStepStop},
    errors::TppCompilerError,
    lexer::{Lexer, ManualDFALexer},
    parser::{Parser, ParserOutput},
    token::Token,
};
use anyhow::anyhow;
use clap::Parser as _;
use log::{debug, info};
use std::{
    collections::HashMap,
    fs::{OpenOptions, exists, read_to_string},
    process::exit,
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

fn lex_and_print_source_code(args: &Args, file: &str, source_code: &str) {
    info!("[lex_and_print_source_code::{file}] Iniciando a análise léxica do arquivo");
    let mut lexer = ManualDFALexer::new(
        source_code,
        args.use_errors_key,
        args.use_identifiers_subset,
        args.omit_eos,
    );

    debug!("[lex_and_print_source_code::{file}] Instanciado o lexer para iniciar a análise");
    for token in lexer.get_tokens() {
        match token {
            Ok(token) => println!("{}", token.kind()),
            Err(mut err) => {
                err.set_file(file.to_string());
                eprintln!("{err}");
            }
        }
    }
    debug!("[lex_and_print_source_code::{file}] Finalizada a tokenização do arquivo");
}

fn lex_source_code<'a>(args: &Args, file: &str, data: &'a str) -> anyhow::Result<Vec<Token<'a>>> {
    info!("[lex_source_files::{file}] Iniciando a análise léxica do arquivo");
    let mut lexer = ManualDFALexer::new(data, args.use_errors_key, args.use_identifiers_subset, args.omit_eos);

    debug!("[lex_source_files::{file}] Instanciado o lexer para iniciar a análise");
    let tokens = lexer.tokenize().map_err(|err| err.with_file(file.to_string()))?;
    debug!("[lex_source_files::{file}] Finalizada a tokenização do arquivo");

    Ok(tokens)
}

fn parse_source_code<'a>(args: &Args, file: &'a str, tokens: Vec<Token<'a>>) -> anyhow::Result<ParserOutput<'a>> {
    info!("[parse_source_code::{file}] Instanciado o parser para iniciar a análise");
    let parser = Parser::new(file, tokens, args.use_errors_key);

    debug!("[parse_source_code::{file}] Iniciando a análise sitática do arquivo");
    let result = parser.into_cst();
    debug!("[parse_source_code::{file}] Finalizado o parsing do arquivo");

    Ok(result)
}

fn run() -> anyhow::Result<()> {
    init_logging()?;
    let args = Args::parse();
    debug!("Leitura de argumentos finalizada.");

    validate_source_files(&args)?;

    let source_codes = acquire_source_codes(&args)?;

    for (file, source_code) in &source_codes {
        if args.stop_at_step.is_some_and(|step| step == CompilerStepStop::Lexer) {
            lex_and_print_source_code(&args, file, source_code);
        } else {
            let tokens = lex_source_code(&args, file, source_code)?;
            let output = parse_source_code(&args, file, tokens)?;

            match output {
                ParserOutput::Failure(errors_bag) => return Err(anyhow!(errors_bag)),
                ParserOutput::PartialFailure(_, mut errors_bag) => {
                    if !errors_bag.is_empty() {
                        let syntax_tree_gen_warning = TppCompilerError::new_parser(
                            "WAR-SYN-NOT-GEN-SYNTAX-TREE",
                            args.use_errors_key,
                            file.to_string(),
                            None,
                            None,
                        );
                        errors_bag.append_error(syntax_tree_gen_warning);
                    }

                    return Err(anyhow!(errors_bag));
                }
                ParserOutput::Success(cst) => {
                    if args.output_cst {
                        println!("{cst:#?}");
                    }
                }
            };
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        exit(1);
    }
}
