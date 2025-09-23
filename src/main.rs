use crate::binary::Binary;
use crate::cli::OutputFormat;

mod binary;
mod cli;
mod command;

fn main() -> anyhow::Result<()> {
    // TODO Also support symbols in `.dynsym` so that shared libraries
    // can be inspected, too.
    let args = cli::Args::parse();
    let binary = Binary::from_file(&args.binary_filename)?;
    let features = command::get_features(
        &binary,
        &args.feature_filter,
        &args.raw_symbol_filter,
        &args.demangled_symbol_filter,
        args.show_symbols,
    )?;

    match args.output_format {
        OutputFormat::List => command::print_list(&features),
        OutputFormat::Table => command::print_table(&features),
        OutputFormat::Json => command::print_json(&features),
    }
}
