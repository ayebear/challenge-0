use anyhow::Result;
use rayon::prelude::*;

fn main() -> Result<()> {
    // generate()?;
    solve()?;
    Ok(())
}

fn generate() -> Result<()> {
    let c1 = std::fs::read("challenge-1.txt")?;
    c1.chunks(4)
        .inspect(|c| {
            assert_eq!(
                c.len(),
                4,
                "please add some spaces/newlines to end of output"
            )
        })
        .map(crc32fast::hash)
        .for_each(|h| println!("{:0>8X}", h));
    Ok(())
}

fn solve() -> Result<()> {
    let results: Vec<String> = std::fs::read_to_string("challenge-0.txt")?
        .par_lines()
        .filter(|line| !line.starts_with('#') && line.len() == 8)
        .map(|line| {
            let h = u32::from_str_radix(line, 16).expect("valid hex");
            crack(h)
        })
        .collect();
    let out = results.iter().fold(String::new(), |a, b| a + &b);
    println!("{out}");
    Ok(())
}

fn crack(target: u32) -> String {
    let chars = (b'a'..=b'z')
        .chain(b'A'..=b'Z')
        .chain(b'0'..=b'9')
        .chain(b"!@#$%^&*()_+-=/\\[]{}|;:'\",.? \n".iter().copied());
    for a in chars.clone() {
        for b in chars.clone() {
            for c in chars.clone() {
                for d in chars.clone() {
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
