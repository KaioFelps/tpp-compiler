//! This module contains the structs needed to parse the commandline arguments.

use clap::Parser;
use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum CompilerStepStop {
    Lexer,
    Parser,
}

#[derive(Parser)]
// command(about) makes a -h or --help by default
#[command(about)]
pub struct Args {
    /// The TPP source files paths.
    pub source_files: Vec<String>,

    /// Whether errors should be logged/printed out as their keys rather than their
    /// actual messages.
    ///
    /// (Disabled by default.)
    #[arg(name = "key", short, long)]
    #[clap(default_value_t = false)]
    pub use_errors_key: bool,

    /// Force the lexer to accept only a subset of allowed letters. This keep the
    /// implementation conformant with Python implementations.
    ///
    /// When disabled, identifiers like `caçamba` or `às9h` are considered valid
    /// identifiers.
    ///
    /// (Disabled by default.)
    #[arg(name = "id-subset", short, long)]
    #[clap(default_value_t = false)]
    pub use_identifiers_subset: bool,

    /// Omits end-of-statement tokens.
    ///
    /// (Disabled by default.)
    #[arg(name = "omit-eos", long)]
    #[clap(default_value_t = false)]
    pub omit_eos: bool,

    /// Stops the compiler at the specified step of compilation process.
    ///
    /// When not specified, the whole compilation process will happen normally.
    #[arg(name = "stop-at", short, long)]
    #[clap(default_value = None)]
    pub stop_at_step: Option<CompilerStepStop>,
}
