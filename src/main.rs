use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::ParallelIterator;
use rayon::prelude::*;

fn main() -> Result<()> {
    // generate0()?;
    generate1()?;
    // solve0()?;
    // solve1()?;
    Ok(())
}

fn generate0() -> Result<()> {
    let mut c1 = std::fs::read("challenge-1.txt")?;
    while c1.len() % 4 != 0 {
        c1.push(b'\n');
    }
    c1.chunks_exact(4)
        .map(crc32fast::hash)
        .for_each(|h| println!("{:0>8X}", h));
    Ok(())
}

fn generate1() -> Result<()> {
    let mut c1 = std::fs::read("challenge-2.txt")?;
    while c1.len() % 4 != 0 {
        c1.push(b'\n');
    }
    let mut r = 42;
    let results: Vec<(String, u64)> = c1
        .chunks_exact(4)
        .map(|buf| {
            r = prng(r);
            let count: u32 = (r & 0xffffffff) as u32;
            r = prng(r);
            let mut fake_count: u64 = r;
            fake_count = (fake_count & 0xffffffff00000000) | (count as u64);
            (buf, count as u64, fake_count)
        })
        .par_bridge()
        .map(|(buf, count, fake_count)| (hash_rounds(count, buf), fake_count))
        .progress_count((c1.len() / 4) as u64)
        .collect();
    for (h, rounds) in results {
        println!("{h}:{rounds}");
    }
    Ok(())
}

fn hash_rounds(count: u64, buf: &[u8]) -> String {
    let mut h = u8_slice_to_u32(&[buf[0], buf[1], buf[2], buf[3]]);
    for _ in 0..count as usize {
        h = crc32fast::hash(&u32_to_u8_slice(h));
    }
    format!("{:0>8X}", h)
}

fn solve0() -> Result<()> {
    eprintln!("Generating all possible CRC32 checksums");
    let table = build_table();
    eprintln!("Applying reverse lookups");
    let mut out: Vec<u8> = Vec::new();
    std::fs::read_to_string("challenge-0.txt")?
        .lines()
        .filter(|line| !line.starts_with('#') && line.len() == 8)
        .for_each(|line| {
            let output = u32::from_str_radix(line, 16).expect("valid hex");
            let input_slice = u32_to_u8_slice(table[output as usize]);
            out.extend(input_slice);
        });
    let out = String::from_utf8(out)?;
    print!("{out}");
    eprintln!("Done!");
    Ok(())
}

// Builds a table of crc32 u32 -> input u32
fn build_table() -> Vec<u32> {
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
