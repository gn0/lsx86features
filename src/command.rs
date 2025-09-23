use itertools::Itertools;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Debug;
use symbolic_common::Name;
use symbolic_demangle::{Demangle, DemangleOptions};
use wildmatch::WildMatch;

use crate::binary::Binary;
use crate::cli::ShowSymbols;

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instruction(String);

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Feature(String);

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcatenatedFeatures(String);

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(String);

pub enum Features {
    Total {
        data: BTreeMap<
            ConcatenatedFeatures,
            BTreeMap<Instruction, usize>,
        >,
    },
    BySymbol {
        data: BTreeMap<
            Symbol,
            BTreeMap<
                ConcatenatedFeatures,
                BTreeMap<Instruction, usize>,
            >,
        >,
    },
}

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
        .map(length)
        .max()
        .filter(|&x| x > min_width)
        .unwrap_or(min_width)
}

pub fn get_features(
    binary: &Binary,
    feature_filter: &[WildMatch],
    raw_symbol_filter: &[WildMatch],
    demangled_symbol_filter: &[WildMatch],
    show_symbols: ShowSymbols,
) -> anyhow::Result<Features> {
    if show_symbols != ShowSymbols::No || !raw_symbol_filter.is_empty()
    {
        let demangle_opts = DemangleOptions::complete();
        let counts = binary
            .instruction_counts_by_symbol()?
            .into_iter()
            .map(|((raw_symbol_name, mnemonic, features), counter)| {
                let symbol_name = match show_symbols {
                    ShowSymbols::No | ShowSymbols::Raw => {
                        Symbol(raw_symbol_name.to_string())
                    }
                    ShowSymbols::Demangled => Symbol(
                        Name::from(raw_symbol_name)
                            .try_demangle(demangle_opts)
                            .to_string(),
                    ),
                };
                let feature_names: Vec<_> = features
                    .iter()
                    .map(|x| Feature(lowercase(x)))
                    .collect();

                (
                    (
                        symbol_name,
                        feature_names,
                        Instruction(lowercase(mnemonic)),
                    ),
                    counter,
                )
            })
            .filter(|((_, features, _), _)| {
                feature_filter.is_empty()
                    || features.iter().any(|Feature(name)| {
                        feature_filter
                            .iter()
                            .any(|pattern| pattern.matches(name))
                    })
            })
            .filter(|((Symbol(raw_symbol_name), _, _), _)| {
                (raw_symbol_filter.is_empty()
                    && demangled_symbol_filter.is_empty())
                    || raw_symbol_filter
                        .iter()
                        .any(|pattern| pattern.matches(raw_symbol_name))
                    || demangled_symbol_filter.iter().any(|pattern| {
                        let demangled = Name::from(raw_symbol_name)
                            .try_demangle(demangle_opts)
                            .to_string();
                        pattern.matches(&demangled)
                    })
            })
            .map(|((symbol, features, mnemonic), counter)| {
                (
                    (
                        symbol,
                        ConcatenatedFeatures(
                            features
                                .into_iter()
                                .map(|Feature(x)| x)
                                .join(","),
                        ),
                        mnemonic,
                    ),
                    counter,
                )
            });
        let mut data = BTreeMap::new();

        for ((symbol_name, feature_names, mnemonic), count) in counts {
            data.entry(symbol_name)
                .or_insert_with(BTreeMap::new)
                .entry(feature_names)
                .or_insert_with(BTreeMap::new)
                .insert(mnemonic, count);
        }

        Ok(Features::BySymbol { data })
    } else {
        let counts = binary
            .instruction_counts()
            .into_iter()
            .map(|((mnemonic, features), count)| {
                let feature_names: Vec<_> = features
                    .iter()
                    .map(|x| Feature(lowercase(x)))
                    .collect();

                (
                    (Instruction(lowercase(mnemonic)), feature_names),
                    count,
                )
            })
            .filter(|((_, features), _)| {
                feature_filter.is_empty()
                    || features.iter().any(|Feature(name)| {
                        feature_filter
                            .iter()
                            .any(|pattern| pattern.matches(name))
                    })
            })
            .map(|((mnemonic, features), count)| {
                (
                    ConcatenatedFeatures(
                        features
                            .into_iter()
                            .map(|Feature(x)| x)
                            .join(","),
                    ),
                    mnemonic,
                    count,
                )
            });
        let mut data = BTreeMap::new();

        for (features, mnemonic, count) in counts {
            data.entry(features)
                .or_insert_with(BTreeMap::new)
                .insert(mnemonic, count);
        }

        Ok(Features::Total { data })
    }
}

pub fn print_list(features: &Features) -> anyhow::Result<()> {
    match features {
        Features::Total { data } => {
            let feature_names = BTreeSet::from_iter(data.keys().map(
                |ConcatenatedFeatures(features)| features.clone(),
            ));

            for name in feature_names.iter() {
                println!("{name}");
            }
        }
        Features::BySymbol { data } => {
            let mut feature_use = BTreeMap::new();

            for (Symbol(symbol), feature_counts) in data.iter() {
                for ConcatenatedFeatures(features) in
                    feature_counts.keys()
                {
                    feature_use
                        .entry(features)
                        .or_insert_with(BTreeSet::new)
                        .insert(symbol);
                }
            }

            for (feature, symbols) in feature_use.iter() {
                println!("Functions that use {feature}:");

                for symbol in symbols.iter() {
                    println!("- {symbol}");
                }

                println!();
            }
        }
    }

    Ok(())
}

pub fn print_table(features: &Features) -> anyhow::Result<()> {
    match features {
        Features::Total { data } => {
            let width_ext = width(
                "Extension",
                data.keys(),
                |ConcatenatedFeatures(features)| features.len(),
            );
            let width_opcode =
                width("Opcode", data.values(), |counts| {
                    counts
                        .keys()
                        .map(|Instruction(mnemonic)| mnemonic.len())
                        .max()
                        .unwrap_or_default()
                });
            let width_count = width("Count", data.values(), |counts| {
                counts
                    .values()
                    .map(|count| {
                        usize::try_from(1 + count.ilog10()).expect(
                            "usize should be at least 32 bits wide",
                        )
                    })
                    .max()
                    .unwrap_or_default()
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

            for (ConcatenatedFeatures(features), counts) in data.iter()
            {
                for (Instruction(mnemonic), count) in counts.iter() {
                    println!(
                        "{0:3$} {1:4$} {2:5$}",
                        features,
                        mnemonic,
                        count,
                        width_ext,
                        width_opcode,
                        width_count
                    );
                }
            }
        }
        Features::BySymbol { data } => {
            let width_sym =
                width("Function", data.keys(), |Symbol(x)| x.len());
            let width_ext =
                width("Extension", data.values(), |counts| {
                    counts
                        .keys()
                        .map(|ConcatenatedFeatures(features)| {
                            features.len()
                        })
                        .max()
                        .unwrap_or_default()
                });
            let width_opcode =
                width("Opcode", data.values(), |counts| {
                    counts
                        .values()
                        .map(|counts| {
                            counts
                                .keys()
                                .map(|Instruction(mnemonic)| {
                                    mnemonic.len()
                                })
                                .max()
                                .unwrap_or_default()
                        })
                        .max()
                        .unwrap_or_default()
                });
            let width_count = width("Count", data.values(), |counts| {
                counts
                    .values()
                    .map(|counts| {
                        counts.values().map(|count| {
                            usize::try_from(1 + count.ilog10()).expect(
                                "usize should be at least 32 bits wide",
                            )
                        }).max().unwrap_or_default()
                    })
                    .max()
                    .unwrap_or_default()
            });

            println!(
                "{0:^4$} {1:^5$} {2:^6$} {3:^7$}",
                "Function",
                "Extension",
                "Opcode",
                "Count",
                width_sym,
                width_ext,
                width_opcode,
                width_count
            );
            println!(
                "{} {} {} {}",
                "-".repeat(width_sym),
                "-".repeat(width_ext),
                "-".repeat(width_opcode),
                "-".repeat(width_count)
            );

            for (Symbol(symbol), feature_counts) in data.iter() {
                for (ConcatenatedFeatures(features), counts) in
                    feature_counts.iter()
                {
                    for (Instruction(mnemonic), count) in counts.iter()
                    {
                        println!(
                            "{0:4$} {1:5$} {2:6$} {3:7$}",
                            symbol,
                            features,
                            mnemonic,
                            count,
                            width_sym,
                            width_ext,
                            width_opcode,
                            width_count
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn print_json(features: &Features) -> anyhow::Result<()> {
    let output = match features {
        Features::Total { data } => serde_json::to_string(data)?,
        Features::BySymbol { data } => serde_json::to_string(data)?,
    };

    println!("{output}");

    Ok(())
}
