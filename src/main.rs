use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::*;
use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
};

fn main() -> Result<()> {
    let flag = env::args().nth(1).unwrap_or_default();
    match flag.as_str() {
        "solve" => {
            let table = build_rainbow_table();
            solve0(&table)?;
            solve1(&table)?;
        }
        "generate" => {
            generate1()?;
            generate0()?;
        }
        _ => {
            eprintln!("Usage: See README.md");
        }
    }
    Ok(())
}

fn generate0() -> Result<()> {
    let mut input = fs::read("challenge-1.txt")?;
    let mut out = BufWriter::new(File::create("challenge-0.txt")?);
    // Pad input to multiple of 4 bytes
    while input.len() % 4 != 0 {
        input.push(b'\n');
    }
    // Write header with challenge description first
    let header = fs::read_to_string("header-0.txt")?;
    writeln!(out, "{header}")?;
    // Calculate single round hashes for input
    let hashes = input.chunks_exact(4).map(crc32fast::hash);
    for hash in hashes {
        writeln!(out, "{:0>8X}", hash)?;
    }
    out.flush()?;
    eprintln!("Wrote challenge-0.txt");
    Ok(())
}

fn generate1() -> Result<()> {
    let mut input = fs::read("challenge-2.txt")?;
    let mut out = BufWriter::new(File::create("challenge-1.txt")?);
    // Pad input to multiple of 4 bytes
    while input.len() % 4 != 0 {
        input.push(b'\n');
    }
    // Write header with challenge description first
    let header = fs::read_to_string("header-1.txt")?;
    writeln!(out, "{header}")?;
    // Calculate multiple round hashes for input
    eprintln!("Generating multiple rounds of CRC32 every 4 bytes");
    // PRNG generate number of rounds per input
    let mut r = 42;
    let rounds: Vec<(u32, u64)> = input
        .chunks_exact(4)
        .map(|buf| {
            r = prng(r);
            let h = u8_slice_to_u32(&[buf[0], buf[1], buf[2], buf[3]]);
            (h, r)
        })
        .collect();
    // Calculate crc32's with correct number of rounds
    let total = (rounds.len() / 4) as u64;
    let results: Vec<(String, u64)> = rounds
        .into_par_iter()
        .map(|(h, count)| (hash_rounds(count, h), count))
        .progress_count(total)
        .collect();
    for (h, count) in results {
        writeln!(out, "{h}:{count}")?;
    }
    out.flush()?;
    eprintln!("Wrote challenge-1.txt");
    Ok(())
}

fn solve0(table: &[u32]) -> Result<()> {
    eprintln!("Applying reverse lookups");
    let mut out = BufWriter::new(File::create("challenge-1.txt")?);
    let input = fs::read_to_string("challenge-0.txt")?;
    let lines = input
        .lines()
        .filter(|line| !line.starts_with('#') && line.len() == 8);
    for line in lines {
        let output = u32::from_str_radix(line, 16).expect("valid hex");
        let input_slice = u32_to_u8_slice(table[output as usize]);
        out.write_all(&input_slice)?;
    }
    out.flush()?;
    eprintln!("Wrote challenge-1.txt");
    Ok(())
}

fn solve1(table: &[u32]) -> Result<()> {
    eprintln!("Parsing challenge file");
    let jobs: Vec<(u64, u32)> = fs::read_to_string("challenge-1.txt")?
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let (hex, count_str) = line.split_at(8);
            let out_hash = u32::from_str_radix(hex, 16).expect("valid hex");
            let count: u64 = count_str[1..].parse().expect("valid number");
            (count, out_hash)
        })
        .collect();
    eprintln!("Applying reverse lookups");
    let total = jobs.len() as u64;
    let results: Vec<[u8; 4]> = jobs
        .par_iter()
        .progress_count(total)
        .map(|&(count, out_hash)| u32_to_u8_slice(unhash_rounds(table, count, out_hash)))
        .collect();
    let mut out = BufWriter::new(File::create("challenge-2.txt")?);
    for chunk in &results {
        out.write_all(chunk)?;
    }
    out.flush()?;
    eprintln!("Wrote challenge-2.txt");
    Ok(())
}

fn hash_rounds(count: u64, mut h: u32) -> String {
    let mut memo = Vec::new();
    for _ in 0..count as usize {
        h = crc32fast::hash(&u32_to_u8_slice(h));
        if !memo.is_empty() && memo[0] == h {
            // eprintln!("period found at {i}, memo size {}", memo.len());
            break;
        }
        memo.push(h);
    }
    let index = count as usize % memo.len();
    let final_hash = memo[index];
    format!("{:0>8X}", final_hash)
}

fn unhash_rounds(table: &[u32], count: u64, mut h: u32) -> u32 {
    let mut memo = Vec::new();
    for _ in 0..count as usize {
        h = table[h as usize];
        if !memo.is_empty() && memo[0] == h {
            // eprintln!("period found at {i}, memo size {}", memo.len());
            break;
        }
        memo.push(h);
    }
    let index = count as usize % memo.len();
    memo[index]
}

// Builds a table of crc32 u32 -> input u32
// Used as a rainbow table to literally crack the hashes
fn build_rainbow_table() -> Vec<u32> {
    eprintln!("Generating crc32->input checksum rainbow table");
    let table = vec![0u32; u32::MAX as usize + 1];
    let count = (u16::MAX as u64) + 1;
    // Multi-threaded
    // Split up into two 16-bit parts to reduce thread over-subscription and progress bar overhead
    (0..=u16::MAX)
        .into_par_iter()
        .progress_count(count)
        .for_each(|a| {
            for b in 0..=u16::MAX {
                let i = ((a as u32) << 16) | (b as u32);
                let h = crc32fast::hash(&u32_to_u8_slice(i));
                unsafe {
                    set_unsync(&table, h as usize, i);
                }
            }
        });
    table
}

// Source: https://stackoverflow.com/a/74020904
unsafe fn set_unsync<T>(vec: &[T], idx: usize, val: T) {
    let start = vec.as_ptr() as *mut T;
    *start.add(idx) = val
}

// Big endian converter
fn u32_to_u8_slice(n: u32) -> [u8; 4] {
    [
        (n >> 24 & 0xFF) as u8,
        (n >> 16 & 0xFF) as u8,
        (n >> 8 & 0xFF) as u8,
        (n & 0xFF) as u8,
    ]
}

// Big endian converter
fn u8_slice_to_u32(b: &[u8; 4]) -> u32 {
    ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32)
}

fn prng(seed: u64) -> u64 {
    const S: u64 = 6364136223846793005;
    seed.wrapping_mul(S).wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converters() {
        assert_eq!(u32_to_u8_slice(0), [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(u32_to_u8_slice(u32::MAX), [0xff, 0xff, 0xff, 0xff]);
        assert_eq!(u32_to_u8_slice(999_999_999), [0x3b, 0x9a, 0xc9, 0xff]);
    }
}
