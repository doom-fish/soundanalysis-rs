# Changelog

## [v0.5.0]

### Added

- `COVERAGE.md` auditing the macOS `SoundAnalysis.framework` headers against the crate surface, including requested-but-unavailable SDK symbols (`SNDetectSoundEventRequest`, `SNAcousticFeaturePrintRequest`, `SNAcousticFeaturePrint`).
- Apple-style alias exports for the primary SoundAnalysis types and protocols: `SNClassifySoundRequest`, `SNClassifierIdentifier`, `SNRequest`, `SNResult`, `SNResultsObserving`, `SNAudioFileAnalyzer`, `SNAudioStreamAnalyzer`, `SNClassification`, `SNClassificationResult`, `SNTimeDurationConstraint`, and `SNTimeRange`.
- `04_apple_aliases` and `05_custom_model_request` examples plus `apple_alias_tests.rs`.

### Changed

- Split the Swift bridge into logical files (`Requests.swift`, `Analyzers.swift`, `Classification.swift`, `Live.swift`) to match the multi-file bridge pattern used by `screencapturekit-rs`.
- Expanded API coverage tests to audit `SNRequest`, `SNResult`, `SNClassifierIdentifier`, and the coverage document itself.

## [v0.4.0]

### Added

- Full `SoundAnalysis.framework` coverage for the public SDK surface.
- `ClassifySoundRequest`, `ClassifierIdentifier`, `TimeDurationConstraint`, and `TimeRange` wrappers for `SNClassifySoundRequest` + `SNTimeDurationConstraint`.
- `AudioFileAnalyzer` exposing request add/remove/remove-all, synchronous analysis, completion-handler analysis, and cancellation.
- `AudioStreamAnalyzer`, `AudioStreamFormat`, `PcmSampleFormat`, and `PcmBuffer` for raw PCM buffer analysis with `SNAudioStreamAnalyzer`.
- `ResultsObserver` + `ResultsObserverFns` to model `SNResultsObserving` from Rust.
- `ClassificationResult::classification_for_identifier()`.
- `03_smoke_surface` example covering the new low-level request, analyzer, and PCM-streaming APIs.
- Expanded API coverage tests for `SNAudioStreamAnalyzer` and `SNTimeDurationConstraint`.

### Changed

- `known_classifications()` now goes through the shared request wrapper surface.
- Example artifacts are written under `target/example-artifacts/` instead of `/tmp`.
- README roadmap now reflects complete framework coverage.

## [0.1.0] - Initial release

### Added

- `classify_file(path)` -> `Vec<ClassificationResult>` — wraps
  `SNAudioFileAnalyzer` + `SNClassifySoundRequest` driven by Apple's
  built-in `SNClassifierIdentifier.version1` model (~300 everyday-sound
  categories). Synchronous, blocking call.
- `known_classifications()` -> `Vec<String>` returns the full label set
  of the built-in classifier so callers can match against it.
- `ClassificationResult { time_start, time_duration, classifications }`
  with `.top()` convenience for the highest-confidence prediction.
- `Classification { identifier, confidence }`.
- `SAError` variants: `InvalidArgument`, `AudioLoadFailed`,
  `RequestCreateFailed`, `AnalysisFailed`, `Unknown { code, message }`.
- 2 examples (`01_classify_file` synthesises speech via `/usr/bin/say`
  and verifies the model returns "speech"; `02_known_classes` lists all
  ~300 categories).
- 5 API-coverage tests (`SNClassifySoundRequest`, `SNAudioFileAnalyzer`,
  `SNClassificationResult`, `SNClassification`, `SNResultsObserving`)
  using the family's Obj-C `@interface` header-parsing harness.
