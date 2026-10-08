//! This module contains the structs needed to parse the commandline arguments.

use clap::Parser;

#[derive(Parser)]
// command(about) makes a -h or --help by default
#[command(about)]
pub struct Args {
    /// The TPP source files paths.
    pub source_files: Vec<String>,

    /// Whether errors should be logged/printed out as their keys rather than their
    /// actual messages.
    #[arg(name = "key", short, long)]
    pub use_errors_key: bool,
}
