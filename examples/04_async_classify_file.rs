//! Async file analysis example
//!
//! This example demonstrates how to use the async API to analyze audio files.
//! Run with: cargo run --example `04_async_classify_file` --features async -- <audio file>

use soundanalysis::async_api::{AsyncAudioFileAnalyzer, AudioStreamAnalysisEvent};
use soundanalysis::ClassifySoundRequest;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(find_test_audio)
        .ok_or("No test audio file found. Please provide an audio file path.")?;

    pollster::block_on(async {
        println!("Async analyzing: {}", audio_path.display());

        let mut analyzer = AsyncAudioFileAnalyzer::new(&audio_path)?;
        let request = ClassifySoundRequest::version1()?;
        let events = analyzer.add_request_stream(&request, 1024)?;

        let reached_end = analyzer.analyze().await?;

        while let Some(event) = events.try_next() {
            match event {
                AudioStreamAnalysisEvent::Result(result) => {
                    if let Some(top) = result.top() {
                        println!(
                            "[{:.2}s+{:.2}s] {} ({:.2})",
                            result.time_start, result.time_duration, top.identifier, top.confidence
                        );
                    }
                }
                AudioStreamAnalysisEvent::Error(error) => eprintln!("analysis error: {error}"),
                AudioStreamAnalysisEvent::Complete => println!("request complete"),
            }
        }

        println!("Async analysis complete (reached end of file: {reached_end})");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

fn find_test_audio() -> Option<PathBuf> {
    ["/tmp/test.wav", "/tmp/test.mp3", "test.wav", "test.mp3"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}
