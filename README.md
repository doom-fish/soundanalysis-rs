# soundanalysis

Safe Rust bindings for Apple's [SoundAnalysis](https://developer.apple.com/documentation/soundanalysis) framework on macOS — on-device sound classification using Apple's built-in `version1` model (~300 everyday sound categories: `"speech"`, `"music"`, `"applause"`, `"dog_bark"`, `"engine_idling"`, `"wind"`, …).

> **Status:** experimental. v0.1 ships file-based classification with Apple's built-in `SNClassifierIdentifierVersion1`. Live audio-buffer streaming via `SNAudioStreamAnalyzer` and custom `MLModel` loading land in v0.2.

## Quick start

```rust,no_run
use soundanalysis::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = classify_file("/tmp/utterance.aiff")?;
    for r in &results {
        if let Some(top) = r.top() {
            println!("[{:.2}s+{:.2}s] {} ({:.2})",
                r.time_start, r.time_duration, top.identifier, top.confidence);
        }
    }
    // Inspect the full label set:
    println!("known classes: {}", known_classifications()?.len());
    Ok(())
}
```

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

- [x] File-based classification (`SNAudioFileAnalyzer` + `SNClassifySoundRequest` + built-in `version1` classifier)
- [x] `known_classifications()` to inspect the label set
- [ ] Live audio-buffer streaming (`SNAudioStreamAnalyzer`)
- [ ] Custom `MLModel` loading (user-trained classifiers)
- [ ] Per-window confidence thresholding builder
- [ ] Async API

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
