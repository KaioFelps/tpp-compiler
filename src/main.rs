mod cli;

use crate::cli::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    println!(
        "Hello, world! usou key: {}; arquivos fontes: {:#?}",
        args.use_errors_key, args.source_files
    );
}
