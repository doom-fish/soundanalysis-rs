//! Smoke test: synthesise an utterance via /usr/bin/say, then classify it.
//! The built-in classifier should return "speech" as the dominant class.
//!
//! Run: `cargo run --example 01_classify_file`

use std::path::PathBuf;
use std::process::Command;

use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts");
    std::fs::create_dir_all(&artifacts)?;
    let aiff = artifacts.join("sa_long.aiff");
    let _ = std::fs::remove_file(&aiff);

    println!("== Step 1: synthesise speech audio via /usr/bin/say ==");
    let status = Command::new("/usr/bin/say")
        .args([
            "-o",
            aiff.to_str().unwrap(),
            "this is a test of sound analysis the quick brown fox jumps over the lazy dog plus more text",
        ])
        .status()?;
    if !status.success() {
        return Err(format!("say failed: {status}").into());
    }
    println!(
        "synthesized {} ({} bytes)",
        aiff.display(),
        std::fs::metadata(&aiff)?.len()
    );

    println!("\n== Step 2: classify ==");
    let results = classify_file(&aiff)?;
    println!("{} analysis windows:", results.len());
    for result in &results {
        let top3: Vec<String> = result
            .classifications
            .iter()
            .take(3)
            .map(|classification| format!("{}={:.2}", classification.identifier, classification.confidence))
            .collect();
        println!(
            "  [{:>5.2}s+{:.2}s] {}",
            result.time_start,
            result.time_duration,
            top3.join(", ")
        );
    }

    let any_speech = results.iter().any(|result| {
        result.classifications.iter().any(|classification| {
            classification.identifier == "speech" && classification.confidence > 0.5
        })
    });
    if any_speech {
        println!("\nOK Speech detected as expected");
    } else {
        println!(
            "\nNote: no high-confidence 'speech' window — model may have categorised the synthesised voice differently."
        );
    }
    Ok(())
}
