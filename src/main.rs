use anyhow::Result;
use rayon::prelude::*;

fn main() -> Result<()> {
    // generate0()?;
    // generate1()?;
    solve()?;
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
    c1.chunks_exact(4)
        .map(rand_hash_rounds)
        .for_each(|(rounds, h)| println!("{rounds}:{h}"));
    Ok(())
}

fn rand_hash_rounds(buf: &[u8]) -> (u64, String) {
    todo!()
}

fn solve() -> Result<()> {
    eprintln!("building table");
    let table = build_table();
    eprintln!("built table");
    let mut out: Vec<u8> = Vec::new();
    std::fs::read_to_string("challenge-0.txt")?
        .lines()
        .filter(|line| !line.starts_with('#') && line.len() == 8)
        .for_each(|line| {
            let output = u32::from_str_radix(line, 16).expect("valid hex");
            let input_slice = u32_to_u8_slice(table[output as usize]);
            // let input = String::from_utf8_lossy(&input_slice).to_string();
            out.extend(input_slice);
        });
    // let out = results.iter().fold(String::new(), |a, b| a + &b);
    // println!("{out}");
    let out = String::from_utf8(out)?;
    print!("{out}");
    eprint!("{out}");
    eprintln!("done");
    Ok(())
}

// Builds a table of crc32 u32 -> input u32
fn build_table() -> Vec<u32> {
    let mut table = vec![0u32; u32::MAX as usize + 1];
    for i in 0..=u32::MAX {
        let h = crc32fast::hash(&u32_to_u8_slice(i));
        table[h as usize] = i;
    }
    table
    // (0..=u32::MAX)
    //     .into_par_iter()
    //     .map(|n| crc32fast::hash(&u32_to_u8_slice(n)))
    //     .collect()
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
fn u8_slice_to_u32(s: &[u8; 4]) -> u32 {
    todo!()
}

fn crack(target: u32) -> String {
    for a in 0..128 {
        for b in 0..128 {
            for c in 0..128 {
                for d in 0..128 {
                    let next = &[a, b, c, d];
                    if target == crc32fast::hash(next) {
                        return String::from_utf8_lossy(next).to_string();
                    }
                }
            }
        }
    }
    // If this panics, need to expand the charset, or just fallback to brute force
    panic!("crc32 not found for target {:0>8X}", target);
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
