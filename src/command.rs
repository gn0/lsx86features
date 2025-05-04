use itertools::Itertools;
use std::fmt::Debug;
use wildmatch::WildMatch;

use crate::binary::Binary;

fn lowercase(value: impl Debug) -> String {
    format!("{value:?}").to_ascii_lowercase()
}

pub fn print_instruction_table(
    binary: &Binary,
    feature_filter: &[WildMatch],
) -> anyhow::Result<()> {
    println!("{:^16} {:^16} {:^6}", "Extension", "Opcode", "Count");
    println!("{:-^16} {:-^16} {:-^6}", "", "", "");

    let rows = binary.instruction_counts().into_iter()
        .map(|((mnemonic, features), counter)| {
            let feature_names: Vec<_> = features.into_iter()
                .map(lowercase)
                .collect();

            ((feature_names, lowercase(mnemonic)), counter)
        })
        .sorted();

    for ((feature_names, mnemonic_name), counter) in rows {
        if !feature_filter.is_empty() {
            let matching_feature = feature_names.iter()
                .any(|name| feature_filter.iter().any(|pattern| {
                    pattern.matches(name)
                }));

            if !matching_feature {
                continue;
            }
        }

        println!(
            "{:16} {:16} {:6}",
            feature_names.join(","),
            mnemonic_name,
            counter
        );
    }

    Ok(())
}

pub fn print_instruction_table_with_symbol(
    binary: &Binary,
    feature_filter: &[WildMatch],
    symbol_filter: &[WildMatch],
) -> anyhow::Result<()> {
    println!(
        "{:^32} {:^16} {:^16} {:^6}",
        "Function", "Extension", "Opcode", "Count"
    );
    println!("{:-^32} {:-^16} {:-^16} {:-^6}", "", "", "", "");

    let rows = binary.instruction_counts_by_symbol()?.into_iter()
        .map(|((symbol, mnemonic, features), counter)| {
            let feature_names: Vec<_> = features.into_iter()
                .map(lowercase)
                .collect();

            ((symbol, feature_names, lowercase(mnemonic)), counter)
        })
        .sorted();

    for ((symbol, feature_names, mnemonic_name), counter) in rows {
        if !feature_filter.is_empty() {
            let matching_feature = feature_names.iter()
                .any(|name| feature_filter.iter().any(|pattern| {
                    pattern.matches(name)
                }));

            if !matching_feature {
                continue;
            }
        }

        if !symbol_filter.is_empty()
            && !symbol_filter.iter().any(|pattern| {
                pattern.matches(symbol)
            })
        {
            continue;
        }

        println!(
            "{:32} {:16} {:16} {:6}",
            symbol,
            feature_names.join(","),
            mnemonic_name,
            counter
        );
    }

    Ok(())
}

pub fn print_by_feature(
    binary: &Binary,
    feature_filter: &[WildMatch],
    symbol_filter: &[WildMatch],
) -> anyhow::Result<()> {
    for (feature, names) in binary.feature_symbols()?.iter() {
        let feature_name = lowercase(feature);

        if !feature_filter.is_empty()
            && !feature_filter.iter().any(|pattern| {
                pattern.matches(&feature_name)
            })
        {
            continue;
        }

        let mut found_first = false;

        for name in names {
            if !symbol_filter.is_empty()
                && !symbol_filter.iter().any(|pattern| {
                    pattern.matches(name)
                })
            {
                continue;
            } else if !found_first {
                println!("Functions that use {feature_name}:");

                found_first = true;
            }

            println!("- {name}");
        }

        if found_first {
            println!();
        }
    }

    Ok(())
}
