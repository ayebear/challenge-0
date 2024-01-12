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
    for a in 0..u8::MAX {
        for b in 0..u8::MAX {
            for c in 0..u8::MAX {
                for d in 0..u8::MAX {
                    let next = &[a, b, c, d];
                    if target == crc32fast::hash(next) {
                        return String::from_utf8_lossy(next).to_string();
                    }
                }
            }
        }
    }
    panic!("crc32 not found for target {target}");
}
