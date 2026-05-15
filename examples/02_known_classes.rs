//! Print every sound category Apple's built-in `version1` classifier can
//! recognise.
//!
//! Run: `cargo run --example 02_known_classes`

use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let labels = known_classifications()?;
    println!("Apple's SNClassifierIdentifier.version1 knows {} categories:\n",
        labels.len());
    for (i, l) in labels.iter().enumerate() {
        if i % 4 == 3 {
            println!("{l:<30}");
        } else {
            print!("{l:<30}");
        }
    }
    println!();
    Ok(())
}
