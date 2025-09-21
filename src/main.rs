use clap::Parser;
use clap::{arg, Arg, ArgGroup, Command};
use std::ffi::OsString;
use wildmatch::WildMatch;

use lsx86features::binary::Binary;
use lsx86features::command;

/// Print the x86 instruction set extensions that are used by a compiled
/// binary.
#[derive(Debug, Parser)]
#[command(version, long_about)]
struct Args {
    /// Print instruction table.
    #[arg(long, short, long_help)]
    instructions: bool,

    /// Print symbols in the instruction table. Requires '-i'.
    #[arg(long, short, long_help, requires = "instructions")]
    symbols: bool,

    /// Comma-separated list of instruction set extensions to include in
    /// the output. Can include wildcard patterns.
    #[arg(long, short = 'F', long_help)]
    feature_filter: Option<String>,

    /// Comma-separated list of symbol names to include in the output.
    /// Can include wildcard patterns.
    #[arg(long, short = 'S', long_help)]
    symbol_filter: Option<String>,

    /// Don't demangle symbol names.
    #[arg(long, short = 'D', long_help)]
    no_demangle: bool,

    // TODO Write the output in JSON.
    #[arg(long, short, long_help)]
    json: bool,

    binary_filename: OsString,
}

fn parse_filter(filter: &str) -> Vec<WildMatch> {
    let mut result = Vec::new();

    for pattern in filter.split(',') {
        result.push(WildMatch::new(pattern));
    }

    result
}

fn main() -> anyhow::Result<()> {
    // TODO Also support symbols in `.dynsym` so that shared libraries
    // can be inspected, too.
    let args = Args::parse();

    let binary = Binary::from_file(&args.binary_filename)?;
    let feature_filter = args
        .feature_filter
        .map(|x| parse_filter(&x))
        .unwrap_or(Vec::new());
    let symbol_filter = args
        .symbol_filter
        .map(|x| parse_filter(&x))
        .unwrap_or(Vec::new());

    match (args.instructions, args.symbols || !symbol_filter.is_empty())
    {
        (true, true) => command::print_instruction_table_with_symbol(
            &binary,
            &feature_filter,
            &symbol_filter,
            !args.no_demangle,
        ),
        (true, false) => {
            command::print_instruction_table(&binary, &feature_filter)
        }
        (false, _) => command::print_by_feature(
            &binary,
            &feature_filter,
            &symbol_filter,
            !args.no_demangle,
        ),
    }
}
