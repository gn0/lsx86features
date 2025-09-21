use itertools::Itertools;
use std::fmt::Debug;
use symbolic_common::Name;
use symbolic_demangle::{Demangle, DemangleOptions};
use wildmatch::WildMatch;

use crate::binary::Binary;

fn lowercase(value: impl Debug) -> String {
    format!("{value:?}").to_ascii_lowercase()
}

fn width<T>(
    title: &str,
    items: impl Iterator<Item = T>,
    length: impl Fn(T) -> usize,
) -> usize {
    let min_width = title.len();

    items
        .map(|x| length(x))
        .max()
        .filter(|&x| x > min_width)
        .unwrap_or(min_width)
}

pub fn print_instruction_table(
    binary: &Binary,
    feature_filter: &[WildMatch],
) -> anyhow::Result<()> {
    let rows: Vec<_> = binary
        .instruction_counts()
        .into_iter()
        .map(|((mnemonic, features), counter)| {
            let feature_names: Vec<_> =
                features.into_iter().map(lowercase).collect();

            ((feature_names, lowercase(mnemonic)), counter)
        })
        .sorted()
        .collect();

    let width_ext =
        width("Extension", rows.iter(), |((feature_names, _), _)| {
            feature_names.join(",").len()
        });
    let width_opcode =
        width("Opcode", rows.iter(), |((_, mnemonic_name), _)| {
            mnemonic_name.len()
        });
    let width_count = width("Count", rows.iter(), |(_, counter)| {
        usize::try_from(1 + counter.ilog10())
            .expect("usize should be at least 32 bits wide")
    });

    println!(
        "{0:^3$} {1:^4$} {2:^5$}",
        "Extension",
        "Opcode",
        "Count",
        width_ext,
        width_opcode,
        width_count
    );
    println!(
        "{} {} {}",
        "-".repeat(width_ext),
        "-".repeat(width_opcode),
        "-".repeat(width_count)
    );

    for ((feature_names, mnemonic_name), counter) in rows {
        if !feature_filter.is_empty() {
            let matching_feature = feature_names.iter().any(|name| {
                feature_filter
                    .iter()
                    .any(|pattern| pattern.matches(name))
            });

            if !matching_feature {
                continue;
            }
        }

        println!(
            "{0:3$} {1:4$} {2:5$}",
            feature_names.join(","),
            mnemonic_name,
            counter,
            width_ext,
            width_opcode,
            width_count
        );
    }

    Ok(())
}

pub fn print_instruction_table_with_symbol(
    binary: &Binary,
    feature_filter: &[WildMatch],
    symbol_filter: &[WildMatch],
    demangle_symbols: bool,
) -> anyhow::Result<()> {
    let demangle_opts = DemangleOptions::complete();

    let rows: Vec<_> = binary
        .instruction_counts_by_symbol()?
        .into_iter()
        .map(|((symbol, mnemonic, features), counter)| {
            let symbol_name: String = if demangle_symbols {
                Name::from(symbol)
                    .try_demangle(demangle_opts)
                    .to_string()
            } else {
                symbol.to_string()
            };
            let feature_names: Vec<_> =
                features.into_iter().map(lowercase).collect();

            ((symbol_name, feature_names, lowercase(mnemonic)), counter)
        })
        .filter(|((_, features, _), _)| {
            feature_filter.is_empty()
                || features.iter().any(|name| {
                    feature_filter
                        .iter()
                        .any(|pattern| pattern.matches(name))
                })
        })
        .filter(|((symbol, _, _), _)| {
            symbol_filter.is_empty()
                || symbol_filter
                    .iter()
                    .any(|pattern| pattern.matches(symbol))
        })
        .sorted()
        .collect();

    let width_func =
        width("Function", rows.iter(), |((symbol, _, _), _)| {
            symbol.len()
        });
    let width_ext = width(
        "Extension",
        rows.iter(),
        |((_, feature_names, _), _)| feature_names.join(",").len(),
    );
    let width_opcode =
        width("Opcode", rows.iter(), |((_, _, mnemonic_name), _)| {
            mnemonic_name.len()
        });
    let width_count = width("Count", rows.iter(), |(_, counter)| {
        usize::try_from(1 + counter.ilog10())
            .expect("usize should be at least 32 bits wide")
    });

    println!(
        "{0:^4$} {1:^5$} {2:^6$} {3:^7$}",
        "Function",
        "Extension",
        "Opcode",
        "Count",
        width_func,
        width_ext,
        width_opcode,
        width_count
    );
    println!(
        "{} {} {} {}",
        "-".repeat(width_func),
        "-".repeat(width_ext),
        "-".repeat(width_opcode),
        "-".repeat(width_count)
    );

    for ((symbol, feature_names, mnemonic_name), counter) in rows {
        println!(
            "{0:4$} {1:5$} {2:6$} {3:7$}",
            symbol,
            feature_names.join(","),
            mnemonic_name,
            counter,
            width_func,
            width_ext,
            width_opcode,
            width_count
        );
    }

    Ok(())
}

pub fn print_by_feature(
    binary: &Binary,
    feature_filter: &[WildMatch],
    symbol_filter: &[WildMatch],
    demangle_symbols: bool,
) -> anyhow::Result<()> {
    let demangle_opts = DemangleOptions::complete();

    for (feature, names) in binary.feature_symbols()?.iter() {
        let feature_name = lowercase(feature);

        if !feature_filter.is_empty()
            && !feature_filter
                .iter()
                .any(|pattern| pattern.matches(&feature_name))
        {
            continue;
        }

        let mut found_first = false;

        for &name in names {
            let symbol_name = if demangle_symbols {
                Name::from(name).try_demangle(demangle_opts).to_string()
            } else {
                name.to_string()
            };

            if !symbol_filter.is_empty()
                && !symbol_filter
                    .iter()
                    .any(|pattern| pattern.matches(&symbol_name))
            {
                continue;
            } else if !found_first {
                println!("Functions that use {feature_name}:");

                found_first = true;
            }

            println!("- {symbol_name}");
        }

        if found_first {
            println!();
        }
    }

    Ok(())
}
