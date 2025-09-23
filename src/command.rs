use itertools::Itertools;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Debug;
use symbolic_common::Name;
use symbolic_demangle::{Demangle, DemangleOptions};
use wildmatch::WildMatch;

use crate::binary::Binary;
use crate::cli::ShowSymbols;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Instruction(String);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Feature(String);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcatenatedFeatures(String);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(String);

pub enum Features {
    Total {
        data: BTreeMap<(ConcatenatedFeatures, Instruction), usize>,
    },
    BySymbol {
        data: BTreeMap<
            Symbol,
            BTreeMap<(ConcatenatedFeatures, Instruction), usize>,
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
                .insert((feature_names, mnemonic), count);
        }

        Ok(Features::BySymbol { data })
    } else {
        let data = binary
            .instruction_counts()
            .into_iter()
            .map(|((mnemonic, features), counter)| {
                let feature_names: Vec<_> = features
                    .iter()
                    .map(|x| Feature(lowercase(x)))
                    .collect();

                (
                    (Instruction(lowercase(mnemonic)), feature_names),
                    counter,
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
            .map(|((mnemonic, features), counter)| {
                (
                    (
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
            })
            .collect();

        Ok(Features::Total { data })
    }
}

pub fn print_list(features: &Features) -> anyhow::Result<()> {
    match features {
        Features::Total { data } => {
            let feature_names = BTreeSet::from_iter(data.keys().map(
                |(ConcatenatedFeatures(features), _)| features.clone(),
            ));

            for name in feature_names.iter() {
                println!("{name}");
            }
        }
        Features::BySymbol { data } => {
            let mut feature_use = BTreeMap::new();

            for (Symbol(symbol), counts) in data.iter() {
                for (ConcatenatedFeatures(features), _) in counts.keys()
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
                data.iter(),
                |((ConcatenatedFeatures(features), _), _)| {
                    features.len()
                },
            );
            let width_opcode = width(
                "Opcode",
                data.iter(),
                |((_, Instruction(mnemonic)), _)| mnemonic.len(),
            );
            let width_count =
                width("Count", data.iter(), |(_, counter)| {
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

            for (
                (ConcatenatedFeatures(features), Instruction(mnemonic)),
                counter,
            ) in data.iter()
            {
                println!(
                    "{0:3$} {1:4$} {2:5$}",
                    features,
                    mnemonic,
                    counter,
                    width_ext,
                    width_opcode,
                    width_count
                );
            }
        }
        Features::BySymbol { data } => {
            let width_sym =
                width("Function", data.keys(), |Symbol(x)| x.len());
            let width_ext =
                width("Extension", data.values(), |counts| {
                    counts
                        .keys()
                        .map(|(ConcatenatedFeatures(features), _)| {
                            features.len()
                        })
                        .max()
                        .unwrap_or_default()
                });
            let width_opcode =
                width("Opcode", data.values(), |counts| {
                    counts
                        .keys()
                        .map(|(_, Instruction(mnemonic))| {
                            mnemonic.len()
                        })
                        .max()
                        .unwrap_or_default()
                });
            let width_count = width("Count", data.values(), |counts| {
                counts
                    .values()
                    .map(|counter| {
                        usize::try_from(1 + counter.ilog10()).expect(
                            "usize should be at least 32 bits wide",
                        )
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

            for (Symbol(symbol), counts) in data.iter() {
                for (
                    (
                        ConcatenatedFeatures(features),
                        Instruction(mnemonic),
                    ),
                    counter,
                ) in counts.iter()
                {
                    println!(
                        "{0:4$} {1:5$} {2:6$} {3:7$}",
                        symbol,
                        features,
                        mnemonic,
                        counter,
                        width_sym,
                        width_ext,
                        width_opcode,
                        width_count
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn print_json(features: &Features) -> anyhow::Result<()> {
    match features {
        Features::Total { data } => todo!(),
        Features::BySymbol { data } => todo!(),
    }
}
