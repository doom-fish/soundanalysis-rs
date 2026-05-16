//! Broad smoke test for the low-level `SoundAnalysis` surface.
//!
//! Run: `cargo run --all-features --example 03_smoke_surface`

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use soundanalysis::prelude::*;

fn synthesize_speech() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts");
    std::fs::create_dir_all(&artifacts)?;
    let audio = artifacts.join("soundanalysis-smoke.aiff");
    let _ = std::fs::remove_file(&audio);

    let status = Command::new("/usr/bin/say")
        .args([
            "-o",
            audio.to_str().unwrap(),
            "hello from sound analysis smoke testing",
        ])
        .status()?;
    if !status.success() {
        return Err(format!("say failed: {status}").into());
    }

    Ok(audio)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio = synthesize_speech()?;

    let mut request = ClassifySoundRequest::version1()?;
    request.set_overlap_factor(0.25)?;
    println!("request overlap factor: {:.2}", request.overlap_factor());

    let labels = request.known_classifications()?;
    println!("request labels: {}", labels.len());

    let constraint = request.window_duration_constraint()?;
    match &constraint {
        TimeDurationConstraint::Enumerated(durations) => {
            println!("window duration constraint: enumerated {} values", durations.len());
            if let Some(first) = durations.first().copied() {
                request.set_window_duration(first)?;
            }
        }
        TimeDurationConstraint::Range(range) => {
            println!(
                "window duration constraint: range start={:.2}s duration={:.2}s",
                range.start_seconds,
                range.duration_seconds
            );
            request.set_window_duration(range.start_seconds.max(0.1))?;
        }
    }
    println!("window duration now: {:.2}s", request.window_duration());

    let manual_enumerated = TimeDurationConstraint::enumerated([0.5, 1.0])?;
    let manual_range = TimeDurationConstraint::range(0.25, 1.5)?;
    println!(
        "manual constraints: {:?} and {:?}",
        manual_enumerated.constraint_type(),
        manual_range.constraint_type()
    );

    let sync_results = Arc::new(Mutex::new(Vec::new()));
    let sync_completions = Arc::new(Mutex::new(0usize));
    let sync_request = request.clone();
    let mut file_analyzer = AudioFileAnalyzer::new(&audio)?;
    file_analyzer.add_request(
        &sync_request,
        ResultsObserverFns::new({
            let sync_results = Arc::clone(&sync_results);
            move |_request, result| {
                sync_results.lock().expect("sync_results poisoned").push(result);
            }
        })
        .on_complete({
            let sync_completions = Arc::clone(&sync_completions);
            move |_request| {
                *sync_completions.lock().expect("sync_completions poisoned") += 1;
            }
        }),
    )?;
    file_analyzer.analyze()?;
    file_analyzer.remove_request(&sync_request);

    let sync_results = sync_results.lock().expect("sync_results poisoned");
    println!("sync analyzer results: {}", sync_results.len());
    if let Some(top) = sync_results.first().and_then(ClassificationResult::top) {
        println!("sync top: {} {:.2}", top.identifier, top.confidence);
        if let Some(found) = sync_results[0].classification_for_identifier(&top.identifier) {
            println!("sync lookup: {} {:.2}", found.identifier, found.confidence);
        }
    }
    println!(
        "sync completions: {}",
        *sync_completions.lock().expect("sync_completions poisoned")
    );
    drop(sync_results);

    let async_hits = Arc::new(Mutex::new(0usize));
    let mut async_analyzer = AudioFileAnalyzer::new(&audio)?;
    let mut async_request = ClassifySoundRequest::with_classifier_identifier(ClassifierIdentifier::Version1)?;
    async_request.set_overlap_factor(0.10)?;
    async_analyzer.add_request(
        &async_request,
        ResultsObserverFns::new({
            let async_hits = Arc::clone(&async_hits);
            move |_request, _result| {
                *async_hits.lock().expect("async_hits poisoned") += 1;
            }
        }),
    )?;
    let reached_end = async_analyzer.analyze_with_completion_handler()?;
    println!("async analyzer reached EOF: {reached_end}");
    println!(
        "async analyzer callbacks: {}",
        *async_hits.lock().expect("async_hits poisoned")
    );
    async_analyzer.cancel_analysis();
    async_analyzer.remove_all_requests();

    let stream_hits = Arc::new(Mutex::new(0usize));
    let format = AudioStreamFormat::new(16_000.0, 1, PcmSampleFormat::Float32, true)?;
    let mut stream_analyzer = AudioStreamAnalyzer::new(format)?;
    let mut stream_request = ClassifySoundRequest::version1()?;
    stream_request.set_overlap_factor(0.20)?;
    stream_analyzer.add_request(
        &stream_request,
        ResultsObserverFns::new({
            let stream_hits = Arc::clone(&stream_hits);
            move |_request, _result| {
                *stream_hits.lock().expect("stream_hits poisoned") += 1;
            }
        }),
    )?;
    let samples: Vec<f32> = (0..16_000)
        .map(|index| (index as f32 / 32.0).sin() * 0.25)
        .collect();
    stream_analyzer.analyze_audio_buffer(
        PcmBuffer::InterleavedF32 {
            samples: &samples,
            channels: 1,
        },
        0,
    )?;
    stream_analyzer.analyze_audio_buffer(
        PcmBuffer::InterleavedF32 {
            samples: &samples,
            channels: 1,
        },
        samples.len() as i64,
    )?;
    stream_analyzer.complete_analysis();
    stream_analyzer.remove_all_requests();
    println!(
        "stream analyzer callbacks: {}",
        *stream_hits.lock().expect("stream_hits poisoned")
    );

    match ClassifySoundRequest::with_model_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/example-artifacts/missing.mlmodelc"),
    ) {
        Ok(_) => println!("custom model request unexpectedly succeeded"),
        Err(error) => println!("custom model request error: {error}"),
    }

    Ok(())
}
