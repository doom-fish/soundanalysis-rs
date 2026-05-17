# Changelog

## [v0.6.1]

### Fixed

- Added SAFETY comments to all unsafe blocks documenting pointer lifetime and validity invariants.
- Added panic handling (`catch_unwind`) to FFI callback trampolines in `live::trampoline()` and `async_api::analyzer_complete_callback()` to prevent panics from unwinding across the FFI boundary.
- Fixed doctests for async API examples to use `ignore` attribute instead of incomplete async context.

## [v0.6.0]

### Added

- **Async API (Tier 1)**: New `async_api` module (gated behind `async` feature) providing `AsyncAudioFileAnalyzer` for non-blocking analysis operations using callback-based FFI.
- `async_api::AsyncAudioFileAnalyzer` for async file analysis with future-based completion.
- Example `04_async_classify_file` demonstrating async API usage with `pollster`.
- Tests in `tests/async_api_tests.rs` covering async analyzer creation and path validation.
- `doom-fish-utils` dependency for async completion utilities.
- `pollster` dev-dependency for running async examples.

## [v0.5.2]

### Added

- Integration coverage for the Requests, Analyzers, Classification, and Live surfaces in `tests/requests_integration.rs`, `tests/analyzers_integration.rs`, `tests/classification_integration.rs`, and `tests/live_integration.rs`.
- Shared speech-synthesis test fixtures under `tests/common/mod.rs` that keep generated artifacts inside `target/test-artifacts/`.

## [v0.5.1]

### Added

- `error_domain()` / `SNErrorDomain()` and `ErrorCode` / `SNErrorCode` for the public `SNError.h` exports (`SNErrorDomain`, `SNErrorCode`).
- Coverage smoke tests for the `SNError.h` surface in `tests/api_coverage.rs` and `tests/apple_alias_tests.rs`.

### Changed

- `COVERAGE.md` and `COVERAGE_AUDIT.md` now report 100.0% coverage for the 46 public macOS `SoundAnalysis` symbols audited from Xcode 26.2.

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
