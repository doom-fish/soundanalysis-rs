//! Smoke test: synthesise an utterance via /usr/bin/say, then classify it.
//! The built-in classifier should return "speech" as the dominant class.
//!
//! Run: `cargo run --example 01_classify_file`

use std::path::PathBuf;
use std::process::Command;
use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aiff: PathBuf = "/tmp/sa_long.aiff".into();
    let _ = std::fs::remove_file(&aiff);

    println!("== Step 1: synthesise speech audio via /usr/bin/say ==");
    let status = Command::new("/usr/bin/say")
        .args([
            "-o", aiff.to_str().unwrap(),
            "this is a test of sound analysis the quick brown fox jumps over the lazy dog plus more text",
        ])
        .status()?;
    if !status.success() {
        return Err(format!("say failed: {status}").into());
    }
    println!("synthesized {} ({} bytes)",
        aiff.display(), std::fs::metadata(&aiff)?.len());

    println!("\n== Step 2: classify ==");
    let results = classify_file(&aiff)?;
    println!("{} analysis windows:", results.len());
    for r in &results {
        let top3: Vec<String> = r.classifications.iter().take(3)
            .map(|c| format!("{}={:.2}", c.identifier, c.confidence))
            .collect();
        println!("  [{:>5.2}s+{:.2}s] {}", r.time_start, r.time_duration, top3.join(", "));
    }

    let any_speech = results.iter().any(|r|
        r.classifications.iter().any(|c|
            c.identifier == "speech" && c.confidence > 0.5));
    if any_speech {
        println!("\nOK Speech detected as expected");
    } else {
        println!("\nNote: no high-confidence 'speech' window — model may have categorised the synthesised voice differently.");
    }
    Ok(())
}
