use anyhow::{anyhow, Context};
use goblin::{elf, Object};
use iced_x86::{
    CpuidFeature, Decoder, DecoderOptions, Instruction, Mnemonic,
};
use std::cmp::Reverse;
use std::collections::{
    BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet,
};
use std::path::Path;

#[derive(Debug)]
pub struct Binary {
    bitness: u32,
    text: Vec<u8>,
    symbols: HashMap<String, (usize, usize)>,
}

impl Binary {
    pub fn from_file<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let data = std::fs::read(path)?;

        let Object::Elf(elf) = Object::parse(&data)? else {
            return Err(anyhow!("Only ELF binaries are supported."));
        };

        Self::from_elf(&elf, &data)
    }

    pub fn from_elf(
        elf: &elf::Elf,
        data: &[u8],
    ) -> anyhow::Result<Self> {
        let bitness = match elf.header.e_machine {
            elf::header::EM_386 => 32,
            elf::header::EM_X86_64 => 64,
            _ => {
                return Err(anyhow!(
                    "Unknown instruction set architecture: {}",
                    elf.header.e_machine
                ))
            }
        };

        // The elf(5) man page lists the sections contained in a binary.
        // The `.text` section contains the executable instructions of
        // the program.
        let Some(text_hdr) =
            elf.section_headers.iter().find(|&section| {
                elf.shdr_strtab.get_at(section.sh_name) == Some(".text")
            })
        else {
            return Err(anyhow!(
                "Binary does not contain a '.text' section"
            ));
        };

        let text_begin = usize::try_from(text_hdr.sh_offset)
            .with_context(|| {
                format!(
                    "The '.text' section has offset {} which is \
                 greater than usize::MAX on this platform",
                    text_hdr.sh_offset
                )
            })?;

        let text_end = text_begin
            + usize::try_from(text_hdr.sh_size).with_context(|| {
                format!(
                    "The '.text' section has size {} which is greater \
                 than usize::MAX on this platform",
                    text_hdr.sh_size
                )
            })?;

        if text_end >= data.len() {
            return Err(anyhow!(
                "Invalid offset + size: {} which is greater than the \
                 binary size, {}",
                text_end,
                data.len()
            ));
        }

        let text_begin_virtual = usize::try_from(text_hdr.sh_addr)
            .with_context(|| {
                format!(
                    "The '.text' section has virtual address {} which \
                 is greater than usize::MAX on this platform",
                    text_hdr.sh_addr
                )
            })?;

        let text_size = text_end - text_begin;
        let text_end_virtual = text_begin_virtual + text_size;

        // Collect symbol addresses and names in increasing order.
        //

        let mut addrs = BinaryHeap::new();

        for sym in elf.syms.iter().chain(elf.dynsyms.iter()) {
            let Some(name) = elf
                .strtab
                .get_at(sym.st_name)
                .or_else(|| elf.dynstrtab.get_at(sym.st_name))
            else {
                continue;
            };

            let addr =
                usize::try_from(sym.st_value).with_context(|| {
                    format!(
                    "Symbol '{}' has address {} which is greater than
                     usize::MAX on this platform",
                    name, sym.st_value
                )
                })?;

            if addr == 0
                || addr < text_begin_virtual
                || addr >= text_end_virtual
            {
                // addr == 0 if the symbol is defined outside of the
                // binary.
                //
                // addr < text_begin_virtual or addr >= text_end_virtual
                // if the symbol is defined outside of the `.text`
                // section.
                continue;
            }

            addrs.push(Reverse((addr, name)));
        }

        let mut symbols = HashMap::new();

        // Extract instructions for each symbol.
        //

        while let Some(Reverse((addr, name))) = addrs.pop() {
            // Relative address within the `.text` section.
            //
            let begin = addr - text_begin_virtual;
            let end = match addrs.peek() {
                Some(Reverse((next, _))) => next - text_begin_virtual,
                None => text_size,
            };

            symbols.insert(name.to_owned(), (begin, end));
        }

        Ok(Binary {
            bitness,
            text: data[text_begin..text_end].to_vec(),
            symbols,
        })
    }

    pub fn instruction_counts(
        &self,
    ) -> HashMap<(Mnemonic, &'static [CpuidFeature]), usize> {
        let mut result = HashMap::new();

        for (mnemonic, features) in
            instructions(&self.text, self.bitness)
        {
            result
                .entry((mnemonic, features))
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }

        result
    }

    pub fn instruction_counts_by_symbol(
        &self,
    ) -> anyhow::Result<
        HashMap<(&str, Mnemonic, &'static [CpuidFeature]), usize>,
    > {
        anyhow::ensure!(
            !self.symbols.is_empty(),
            "No symbols found in the '.text' section, the binary may \
             have been stripped"
        );

        let mut result = HashMap::new();

        for (name, &(begin, end)) in self.symbols.iter() {
            for (mnemonic, features) in
                instructions(&self.text[begin..end], self.bitness)
            {
                result
                    .entry((name.as_str(), mnemonic, features))
                    .and_modify(|counter| *counter += 1)
                    .or_insert(1);
            }
        }

        Ok(result)
    }

    pub fn symbol_features(
        &self,
    ) -> anyhow::Result<BTreeMap<&str, Vec<CpuidFeature>>> {
        anyhow::ensure!(
            !self.symbols.is_empty(),
            "No symbols found in the '.text' section, the binary may \
             have been stripped"
        );

        let mut result = BTreeMap::new();

        for (name, &(begin, end)) in self.symbols.iter() {
            let mut sym_features = HashSet::new();

            for (mnemonic, features) in
                instructions(&self.text[begin..end], self.bitness)
            {
                sym_features.extend(features);
            }

            result.insert(
                name.as_str(),
                sym_features.into_iter().collect(),
            );
        }

        Ok(result)
    }

    pub fn feature_symbols(
        &self,
    ) -> anyhow::Result<BTreeMap<CpuidFeature, Vec<&str>>> {
        let symbol_features = self.symbol_features()?;

        let mut result: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();

        for (name, features) in symbol_features.into_iter() {
            for feature in features {
                if let Some(symbols) = result.get_mut(&feature) {
                    (*symbols).insert(name);
                } else {
                    result.insert(feature, BTreeSet::from([name]));
                }
            }
        }

        Ok(BTreeMap::from_iter(result.into_iter().map(
            |(feature, symbols)| {
                (feature, symbols.into_iter().collect())
            },
        )))
    }
}

fn instructions(
    data: &[u8],
    bitness: u32,
) -> Vec<(Mnemonic, &'static [CpuidFeature])> {
    let mut decoder = Decoder::new(bitness, data, DecoderOptions::NONE);
    let mut instruction = Instruction::default();
    let mut result = Vec::new();

    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);

        result.push((
            instruction.op_code().mnemonic(),
            instruction.cpuid_features(),
        ));
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instructions_sse() {
        #[rustfmt::skip]
        let add_arrays_sse: &[u8] = &[
            0x0f, 0x28, 0x06,  // movaps xmm0,XMMWORD PTR [rsi]
            0x0f, 0x28, 0x0a,  // movaps xmm1,XMMWORD PTR [rdx]
            0x0f, 0x58, 0xc1,  // addps xmm0,xmm1
            0x0f, 0x29, 0x07,  // movaps XMMWORD PTR [rdi],xmm0
            0xc3,              // ret
            0x0f, 0x1f, 0x00,  // nop DWORD PTR [rax]
        ];
        let result: Vec<(Mnemonic, &[CpuidFeature])> = vec![
            (Mnemonic::Movaps, &[CpuidFeature::SSE]),
            (Mnemonic::Movaps, &[CpuidFeature::SSE]),
            (Mnemonic::Addps, &[CpuidFeature::SSE]),
            (Mnemonic::Movaps, &[CpuidFeature::SSE]),
            (Mnemonic::Ret, &[CpuidFeature::X64]),
            (Mnemonic::Nop, &[CpuidFeature::MULTIBYTENOP]),
        ];

        assert_eq!(instructions(&add_arrays_sse, 64), result);
    }

    #[test]
    fn instructions_avx2() {
        #[rustfmt::skip]
        let add_arrays_avx2: &[u8] = &[
            0xc5, 0xfc, 0x77,        // vzeroall
            0xc5, 0xfc, 0x28, 0x06,  // vmovaps ymm0,YMMWORD PTR [rsi]
            0xc5, 0xfc, 0x28, 0x0a,  // vmovaps ymm1,YMMWORD PTR [rdx]
            0xc5, 0xfc, 0x58, 0xd1,  // vaddps ymm2,ymm0,ymm1
            0xc5, 0xfc, 0x29, 0x17,  // vmovaps YMMWORD PTR [rdi],ymm2
            0xc3,                    // ret
            0x66, 0x2e, 0x0f, 0x1f, 0x84, 0x00, 0x00, // cs nop WORD PTR [rax+rax*1+0x0]
            0x00, 0x00, 0x00,
            0x66, 0x90,              // xchg ax,ax
        ];
        let result: Vec<(Mnemonic, &[CpuidFeature])> = vec![
            (Mnemonic::Vzeroall, &[CpuidFeature::AVX]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX]),
            (Mnemonic::Vaddps, &[CpuidFeature::AVX]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX]),
            (Mnemonic::Ret, &[CpuidFeature::X64]),
            (Mnemonic::Nop, &[CpuidFeature::MULTIBYTENOP]),
            (Mnemonic::Nop, &[CpuidFeature::INTEL8086]),
        ];

        assert_eq!(instructions(&add_arrays_avx2, 64), result);
    }

    #[test]
    fn instructions_avx512() {
        #[rustfmt::skip]
        let add_arrays_avx512: &[u8] = &[
            0xc5, 0xfc, 0x77,                    // vzeroall
            0x62, 0xf1, 0x7c, 0x48, 0x28, 0x06,  // vmovaps zmm0,ZMMWORD PTR [rsi]
            0x62, 0xf1, 0x7c, 0x48, 0x28, 0x0a,  // vmovaps zmm1,ZMMWORD PTR [rdx]
            0x62, 0xf1, 0x7c, 0x48, 0x58, 0xd1,  // vaddps zmm2,zmm0,zmm1
            0x62, 0xf1, 0x7c, 0x48, 0x29, 0x17,  // vmovaps ZMMWORD PTR [rdi],zmm2
            0xc3,                                // ret
        ];
        let result: Vec<(Mnemonic, &[CpuidFeature])> = vec![
            (Mnemonic::Vzeroall, &[CpuidFeature::AVX]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX512F]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX512F]),
            (Mnemonic::Vaddps, &[CpuidFeature::AVX512F]),
            (Mnemonic::Vmovaps, &[CpuidFeature::AVX512F]),
            (Mnemonic::Ret, &[CpuidFeature::X64]),
        ];

        assert_eq!(instructions(&add_arrays_avx512, 64), result);
    }
}
