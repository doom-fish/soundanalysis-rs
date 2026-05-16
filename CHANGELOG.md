# Changelog

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
