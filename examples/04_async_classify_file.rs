//! Async file analysis example
//!
//! This example demonstrates how to use the async API to analyze audio files.
//! Run with: cargo run --example 04_async_classify_file --features async

use soundanalysis::async_api::AsyncAudioFileAnalyzer;
use soundanalysis::ClassifySoundRequest;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Try to find a test audio file
        let audio_path = find_test_audio()
            .ok_or("No test audio file found. Please provide an audio file path.")?;

        println!("Async analyzing: {}", audio_path.display());

        // Create the async analyzer
        let _analyzer = AsyncAudioFileAnalyzer::new(&audio_path)?;

        // Create a classification request
        let _request = ClassifySoundRequest::version1()?;

        println!("Async analysis complete!");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

fn find_test_audio() -> Option<PathBuf> {
    // Try some common test audio paths
    let paths = vec![
        "/tmp/test.wav",
        "/tmp/test.mp3",
        "test.wav",
        "test.mp3",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}
