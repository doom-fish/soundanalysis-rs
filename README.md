# soundanalysis

Safe Rust bindings for Apple's [SoundAnalysis](https://developer.apple.com/documentation/soundanalysis) framework on macOS.

> **Status:** v0.5 audits the public macOS `SoundAnalysis.framework` surface against the Xcode 26.2 SDK, adds Apple-style aliases (`SNClassifySoundRequest`, `SNClassifierIdentifier`, `SNRequest`, `SNResult`, `SNTimeRange`), and documents the full mapping in [`COVERAGE.md`](COVERAGE.md). Requested symbols that are absent from the current macOS SDK (`SNDetectSoundEventRequest`, `SNAcousticFeaturePrintRequest`, `SNAcousticFeaturePrint`) are called out there as skipped.

## Quick start

```rust,no_run
use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = classify_file("target/utterance.aiff")?;
    for result in &results {
        if let Some(top) = result.top() {
            println!(
                "[{:.2}s+{:.2}s] {} ({:.2})",
                result.time_start,
                result.time_duration,
                top.identifier,
                top.confidence
            );
        }
    }

    println!("known classes: {}", known_classifications()?.len());
    Ok(())
}
```

## Low-level framework surface

```rust,no_run
use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = ClassifySoundRequest::version1()?;
    request.set_overlap_factor(0.25)?;

    if let TimeDurationConstraint::Range(range) = request.window_duration_constraint()? {
        request.set_window_duration(range.start_seconds.max(0.5))?;
    }

    let mut analyzer = AudioFileAnalyzer::new("target/utterance.aiff")?;
    analyzer.add_request(
        &request,
        ResultsObserverFns::new(|_request, result| {
            if let Some(top) = result.top() {
                println!("{} {:.2}", top.identifier, top.confidence);
            }
        }),
    )?;
    analyzer.analyze()?;
    Ok(())
}
```

For real-time use, `AudioStreamAnalyzer` accepts interleaved or planar PCM buffers (`f32`, `f64`, `i16`, `i32`) and `start_live_classification()` keeps the high-level microphone convenience API. If you prefer Apple naming, the crate now re-exports `SNClassifySoundRequest`, `SNClassifierIdentifier`, `SNAudioFileAnalyzer`, `SNAudioStreamAnalyzer`, `SNClassificationResult`, `SNClassification`, `SNTimeDurationConstraint`, `SNTimeRange`, `SNRequest`, `SNResult`, and `SNResultsObserving` alongside the ergonomic Rust names.

## Examples

- `cargo run --example 01_classify_file`
- `cargo run --example 02_known_classes`
- `cargo run --all-features --example 03_smoke_surface`
- `cargo run --example 04_apple_aliases`
- `cargo run --example 05_custom_model_request`

## Pipeline composition

```text
screencapturekit-rs ──► system audio ──► soundanalysis ──► event timeline
                                                             │
                                                             ▼
                                                    speech ──► transcript
                                                             │
                                                             ▼
                                                    naturallanguage ──► entities
                                                             │
                                                             ▼
                                                    foundation-models
                                                    ("summarise the meeting,
                                                     flag the cough at 3:42")
```

Pairs naturally with [`screencapturekit`](https://github.com/doom-fish/screencapturekit-rs) (system audio capture) and [`speech`](https://github.com/doom-fish/speech-rs) for full audio-understanding pipelines.

## Roadmap

- [x] File-based classification (`SNAudioFileAnalyzer`)
- [x] Request tuning (`overlapFactor`, `windowDuration`, `windowDurationConstraint`)
- [x] Built-in classifier metadata (`knownClassifications`, `ClassifierIdentifier::Version1`)
- [x] Custom Core ML request creation (`SNClassifySoundRequest(mlModel:)`)
- [x] File analyzer request management (`add/remove/removeAll`, sync + completion-handler analysis, cancel)
- [x] Stream analyzer request management (`add/remove/removeAll`, raw PCM `analyzeAudioBuffer`, `completeAnalysis`)
- [x] Results observing callbacks and `ClassificationResult::classification_for_identifier`
- [x] High-level microphone convenience (`start_live_classification`)

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
