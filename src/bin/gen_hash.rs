use std::env;
use std::io::{self, Read};
use std::time::Instant;

struct XorShift64(u64);

impl XorShift64 {
    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    #[inline(always)]
    fn next_usize_range(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u64() as usize % (max - min + 1))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HashFormulaKind {
    ThreeByte,
    FourByte,
    FiveByteWithByte6,
}

struct HashSolution {
    formula: HashFormulaKind,
    table_size: usize,
    mask: usize,
    shift: usize,
    m_len: usize,
    m_b0: usize,
    m_b1: usize,
    m_b2: usize,
    m_bmid: usize,
    m_blast: usize,
}

fn solve_perfect_hash(kws: &[String], case_insensitive: bool) -> Option<HashSolution> {
    let kw_count = kws.len();
    let min_pow2 = (kw_count.next_power_of_two()).max(32);
    let mut table_sizes = vec![min_pow2, min_pow2 * 2, min_pow2 * 4];
    if min_pow2 > 32 {
        table_sizes.insert(0, min_pow2 / 2);
    }
    table_sizes.retain(|&sz| sz >= kw_count);

    let prepared: Vec<Vec<u8>> = kws
        .iter()
        .map(|s| {
            if case_insensitive {
                s.as_bytes().iter().map(|&b| b | 0x20).collect()
            } else {
                s.as_bytes().to_vec()
            }
        })
        .collect();

    for &sz in &table_sizes {
        let mask = sz - 1;
        let mut rng = XorShift64(0x853c49e6748fea9b ^ (sz as u64));

        for _ in 0..1_000_000 {
            let shift = rng.next_usize_range(1, 3);
            let m_len = rng.next_usize_range(1, 10_000);
            let m_b0 = rng.next_usize_range(1, 10_000);
            let m_bmid = rng.next_usize_range(1, 10_000);
            let m_blast = rng.next_usize_range(1, 10_000);

            let mut occupied = [false; 2048];
            let mut collision = false;

            for cand in &prepared {
                let len = cand.len();
                let b0 = cand[0] as usize;
                let bmid = cand[len / 2] as usize;
                let blast = cand[len - 1] as usize;
                let val = len
                    .wrapping_mul(m_len)
                    .wrapping_add(b0.wrapping_mul(m_b0))
                    .wrapping_add(bmid.wrapping_mul(m_bmid))
                    .wrapping_add(blast.wrapping_mul(m_blast));
                let h = (val ^ (val >> shift)) & mask;
                if occupied[h] {
                    collision = true;
                    break;
                }
                occupied[h] = true;
            }

            if !collision {
                return Some(HashSolution {
                    formula: HashFormulaKind::ThreeByte,
                    table_size: sz,
                    mask,
                    shift,
                    m_len,
                    m_b0,
                    m_b1: 0,
                    m_b2: 0,
                    m_bmid,
                    m_blast,
                });
            }
        }

        for _ in 0..2_000_000 {
            let shift = rng.next_usize_range(1, 3);
            let m_len = rng.next_usize_range(1, 10_000);
            let m_b0 = rng.next_usize_range(1, 10_000);
            let m_b1 = rng.next_usize_range(1, 10_000);
            let m_bmid = rng.next_usize_range(1, 10_000);
            let m_blast = rng.next_usize_range(1, 10_000);

            let mut occupied = [false; 2048];
            let mut collision = false;

            for cand in &prepared {
                let len = cand.len();
                let b0 = cand[0] as usize;
                let b1 = cand[1 % len] as usize;
                let bmid = cand[len / 2] as usize;
                let blast = cand[len - 1] as usize;
                let val = len
                    .wrapping_mul(m_len)
                    .wrapping_add(b0.wrapping_mul(m_b0))
                    .wrapping_add(b1.wrapping_mul(m_b1))
                    .wrapping_add(bmid.wrapping_mul(m_bmid))
                    .wrapping_add(blast.wrapping_mul(m_blast));
                let h = (val ^ (val >> shift)) & mask;
                if occupied[h] {
                    collision = true;
                    break;
                }
                occupied[h] = true;
            }

            if !collision {
                return Some(HashSolution {
                    formula: HashFormulaKind::FourByte,
                    table_size: sz,
                    mask,
                    shift,
                    m_len,
                    m_b0,
                    m_b1,
                    m_b2: 0,
                    m_bmid,
                    m_blast,
                });
            }
        }

        for _ in 0..2_000_000 {
            let shift = rng.next_usize_range(1, 3);
            let m_len = rng.next_usize_range(1, 10_000);
            let m_b0 = rng.next_usize_range(1, 10_000);
            let m_b1 = rng.next_usize_range(1, 10_000);
            let m_b2 = rng.next_usize_range(1, 10_000);
            let m_bmid = rng.next_usize_range(1, 10_000);
            let m_blast = rng.next_usize_range(1, 10_000);

            let mut occupied = [false; 2048];
            let mut collision = false;

            for cand in &prepared {
                let len = cand.len();
                let b0 = cand[0] as usize;
                let b1 = cand[1 % len] as usize;
                let b6 = if len > 6 { cand[6] as usize } else { 0 };
                let bmid = cand[len / 2] as usize;
                let blast = cand[len - 1] as usize;
                let val = len
                    .wrapping_mul(m_len)
                    .wrapping_add(b0.wrapping_mul(m_b0))
                    .wrapping_add(b1.wrapping_mul(m_b1))
                    .wrapping_add(b6.wrapping_mul(m_b2))
                    .wrapping_add(bmid.wrapping_mul(m_bmid))
                    .wrapping_add(blast.wrapping_mul(m_blast));
                let h = (val ^ (val >> shift)) & mask;
                if occupied[h] {
                    collision = true;
                    break;
                }
                occupied[h] = true;
            }

            if !collision {
                return Some(HashSolution {
                    formula: HashFormulaKind::FiveByteWithByte6,
                    table_size: sz,
                    mask,
                    shift,
                    m_len,
                    m_b0,
                    m_b1,
                    m_b2,
                    m_bmid,
                    m_blast,
                });
            }
        }
    }

    None
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut case_insensitive = false;
    let mut kws: Vec<String> = Vec::new();

    for arg in &args {
        if arg == "-i" || arg == "--ignore-case" {
            case_insensitive = true;
        } else if !arg.starts_with('-') {
            kws.push(arg.clone());
        }
    }

    if kws.is_empty() {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() {
            for line in buffer.lines() {
                for token in line.split_whitespace() {
                    let clean = token.trim_matches(|c| c == ',' || c == '"' || c == '\'' || c == ';');
                    if !clean.is_empty() && !kws.iter().any(|k| k == clean) {
                        kws.push(clean.to_string());
                    }
                }
            }
        }
    }

    kws.sort();
    kws.dedup();

    if kws.is_empty() {
        eprintln!("Usage: gen_hash [-i|--ignore-case] <keyword1> <keyword2> ... or via stdin");
        std::process::exit(1);
    }

    let kw_count = kws.len();
    let min_len = kws.iter().map(|k| k.len()).min().unwrap_or(0);
    let max_len = kws.iter().map(|k| k.len()).max().unwrap_or(0);

    let start = Instant::now();
    let sol = solve_perfect_hash(&kws, case_insensitive);
    let elapsed = start.elapsed();

    let Some(sol) = sol else {
        eprintln!("Failed to find perfect hash for {} keywords in time", kw_count);
        std::process::exit(1);
    };

    println!("// Solved in {:.2?}", elapsed);
    println!("pub const KEYWORD_COUNT: usize = {};", kw_count);
    println!("const KW_SLOT_MASK: usize = {};", sol.mask);
    println!("const KW_EMPTY: u8 = 255;\n");

    println!("pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [");
    for kw in &kws {
        println!("    \"{}\",", kw);
    }
    println!("];\n");

    println!("pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);");
    println!("pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);");
    println!("pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);\n");

    println!("const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);");
    println!("const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);");
    println!("const _: () = assert!(KW_MIN_LEN >= {} && KW_MAX_LEN <= {});\n", min_len, max_len);

    println!("#[inline(always)]");
    println!("const fn kw_hash(cand: &[u8], len: usize) -> usize {{");
    if case_insensitive {
        println!("    let b0 = (cand[0] | 0x20) as usize;");
        println!("    let bmid = (cand[len / 2] | 0x20) as usize;");
        println!("    let blast = (cand[len - 1] | 0x20) as usize;");
    } else {
        println!("    let b0 = cand[0] as usize;");
        println!("    let bmid = cand[len / 2] as usize;");
        println!("    let blast = cand[len - 1] as usize;");
    }

    match sol.formula {
        HashFormulaKind::ThreeByte => {
            println!("    let val = len.wrapping_mul({})", sol.m_len);
            println!("        .wrapping_add(b0.wrapping_mul({}))", sol.m_b0);
            println!("        .wrapping_add(bmid.wrapping_mul({}))", sol.m_bmid);
            println!("        .wrapping_add(blast.wrapping_mul({}));", sol.m_blast);
        }
        HashFormulaKind::FourByte => {
            if case_insensitive {
                println!("    let b1 = (cand[1] | 0x20) as usize;");
            } else {
                println!("    let b1 = cand[1] as usize;");
            }
            println!("    let val = len.wrapping_mul({})", sol.m_len);
            println!("        .wrapping_add(b0.wrapping_mul({}))", sol.m_b0);
            println!("        .wrapping_add(b1.wrapping_mul({}))", sol.m_b1);
            println!("        .wrapping_add(bmid.wrapping_mul({}))", sol.m_bmid);
            println!("        .wrapping_add(blast.wrapping_mul({}));", sol.m_blast);
        }
        HashFormulaKind::FiveByteWithByte6 => {
            if case_insensitive {
                println!("    let b1 = (cand[1] | 0x20) as usize;");
                println!("    let b6 = if len > 6 {{ (cand[6] | 0x20) as usize }} else {{ 0 }};");
            } else {
                println!("    let b1 = cand[1] as usize;");
                println!("    let b6 = if len > 6 {{ cand[6] as usize }} else {{ 0 }};");
            }
            println!("    let val = len.wrapping_mul({})", sol.m_len);
            println!("        .wrapping_add(b0.wrapping_mul({}))", sol.m_b0);
            println!("        .wrapping_add(b1.wrapping_mul({}))", sol.m_b1);
            println!("        .wrapping_add(b6.wrapping_mul({}))", sol.m_b2);
            println!("        .wrapping_add(bmid.wrapping_mul({}))", sol.m_bmid);
            println!("        .wrapping_add(blast.wrapping_mul({}));", sol.m_blast);
        }
    }

    println!("    (val ^ (val >> {})) & KW_SLOT_MASK", sol.shift);
    println!("}}\n");

    println!("const KW_TABLE: [u8; {}] = {{", sol.table_size);
    println!("    let mut table = [KW_EMPTY; {}];", sol.table_size);
    println!("    let mut i = 0;");
    println!("    while i < KEYWORD_COUNT {{");
    println!("        let name = KEYWORD_NAMES[i].as_bytes();");
    println!("        let len = KEYWORD_LENS[i] as usize;");
    println!("        assert!(name.len() == len);");
    println!("        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);");
    println!("        let h = kw_hash(name, len);");
    println!("        assert!(table[h] == KW_EMPTY);");
    println!("        table[h] = i as u8;");
    println!("        i += 1;");
    println!("    }}");
    println!("    table");
    println!("}};");
}
