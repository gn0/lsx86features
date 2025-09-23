use clap::{arg, command, ArgGroup};
use wildmatch::WildMatch;

#[derive(Debug)]
pub struct Args {
    pub output_format: OutputFormat,
    pub show_symbols: ShowSymbols,
    pub feature_filter: Vec<WildMatch>,
    pub raw_symbol_filter: Vec<WildMatch>,
    pub demangled_symbol_filter: Vec<WildMatch>,
    pub binary_filename: String,
}

#[derive(Debug)]
pub enum OutputFormat {
    List,
    Table,
    Json,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ShowSymbols {
    No,
    Raw,
    Demangled,
}

impl Args {
    pub fn parse() -> Self {
        let matches = command!()
            .arg(arg!(-l --list "Print output as list"))
            .arg(arg!(-t --table "Print output as table"))
            .arg(arg!(-j --json "Print output as JSON"))
            .group(
                ArgGroup::new("output-format")
                    .args(["list", "table", "json"]),
            )
            .arg(arg!(
                    -s --"show-symbol"
                    "Include raw symbol names in output"
            ))
            .arg(arg!(
                    -d --"show-demangled"
                    "Include demangled symbol names in output"
            ))
            .group(
                ArgGroup::new("symbols")
                    .args(["show-symbol", "show-demangled"]),
            )
            .arg(arg!(
                    -F --"feature-filter" <STRING>
                    "Comma-separated list of extension sets to include \
                     in the output (can include wildcards)"
            ))
            .arg(arg!(
                    -S --"raw-symbol-filter" <STRING>
                    "Comma-separated list of raw symbol names to \
                     include in the output (can include wildcards)"
            ))
            .arg(arg!(
                    -D --"demangled-symbol-filter" <STRING>
                    "Comma-separated list of demangled symbol names to \
                     include in the output (can include wildcards)"
            ))
            .arg(
                arg!(<BINARY_FILENAME> "Filename of binary to inspect"),
            )
            .get_matches();

        let output_format =
            if *matches.get_one("list").expect("should be Some") {
                OutputFormat::List
            } else if *matches.get_one("json").expect("should be Some")
            {
                OutputFormat::Json
            } else {
                // Default:
                OutputFormat::Table
            };

        let show_symbols =
            if *matches.get_one("show-symbol").expect("should be Some")
            {
                ShowSymbols::Raw
            } else if *matches
                .get_one("show-demangled")
                .expect("should be Some")
            {
                ShowSymbols::Demangled
            } else {
                // Default:
                ShowSymbols::No
            };

        let feature_filter = matches
            .get_one::<String>("feature-filter")
            .map(|x| parse_filter(x))
            .unwrap_or_default();
        let raw_symbol_filter = matches
            .get_one::<String>("raw-symbol-filter")
            .map(|x| parse_filter(x))
            .unwrap_or_default();
        let demangled_symbol_filter = matches
            .get_one::<String>("demangled-symbol-filter")
            .map(|x| parse_filter(x))
            .unwrap_or_default();
        let binary_filename = matches
            .get_one::<String>("BINARY_FILENAME")
            .cloned()
            .expect("required");

        Self {
            output_format,
            show_symbols,
            feature_filter,
            raw_symbol_filter,
            demangled_symbol_filter,
            binary_filename,
        }
    }
}

fn parse_filter(filter: &str) -> Vec<WildMatch> {
    let mut result = Vec::new();

    for pattern in filter.split(',') {
        result.push(WildMatch::new(pattern));
    }

    result
}
