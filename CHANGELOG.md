# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.2] - 2026-10-26

### Fixed

- Update the test assertions for Clippy 1.99 on hosted runners.

## [0.7.1] - 2026-10-26

### Fixed

- Build the Swift bridge with the default SwiftPM build engine and use its reported product directory.

## [0.7.0] - 2026-09-24

### Security

- The live microphone session's callback context was freed right after
  the engine stopped while its observer still held the raw pointer and the
  request stayed registered. The Swift observer now retains the context
  through doom-fish-utils' `CallbackContext`, stopping removes every request
  before the session is released, and late callbacks are ignored.

### Fixed

- `AsyncAudioFileAnalyzer::analyze()` was a stub that created an analyzer
  and reported success without analyzing. It now runs
  `analyzeWithCompletionHandler:` for the analyzer's requests and resolves
  to `didReachEndOfFile`; dropping the future before it resolves cancels the
  analysis.
- `AudioStreamAnalyzer` checks PCM buffer sizes with checked arithmetic and
  rejects empty buffers and buffers whose frame count or per-buffer byte
  size does not fit an `AVAudioPCMBuffer`; the bridge no longer traps on
  them or copies a wrapped byte count.
- The live observer reports analysis failures and completion, and starting
  without an audio input device returns an error instead of hitting an
  `AVAudioEngine` exception.
- The file and stream analyzer boxes record observer errors under a lock,
  and a file analyzer rejects a second analysis or a new request while one
  is running.
- Stream capacity zero is rejected with `SAError::InvalidArgument` instead
  of panicking.
- The crate builds with `--no-default-features`. The `observer` module and
  the `ResultsObserver`, `ResultsObserverFns`, `SNResult`, `AnalysisResult`
  and `SNResultsObserving` re-exports only exist with `analyze_file`, like
  the analyzers they serve (without it the crate did not compile at all).
- The request and analyzer constructors return `SAError::Unknown` instead
  of panicking when the bridge reports success with a null handle.

### Changed

- **Breaking:** `start_live_classification` takes a `FnMut(LiveEvent)`
  handler; `LiveEvent` is `Update(StreamUpdate)`, `Failed(SAError)` or
  `Completed`.
- **Breaking:** `AsyncAudioFileAnalyzer::new` opens the audio file, so it
  fails for a missing file. The analyzer dereferences to `AudioFileAnalyzer`,
  `analyze()` takes `&mut self`, and `AnalyzeFileFuture` has a lifetime and
  resolves to `Result<bool, SAError>`.
- **Breaking:** `ClassifySoundRequest` no longer implements `Clone`. It has
  `&mut` setters, but a clone only retained the same
  `SNClassifySoundRequest`, so changing the clone changed the original and
  every analyzer using it; the class can't be copied. Create another
  request instead.
- **Breaking (raw FFI):** `sa_audio_file_analyzer_analyze_async` takes the
  analyzer handle and reports status, `didReachEndOfFile` and an error
  message; `sa_stream_start` takes error and completion callbacks and
  context retain/release callbacks.
- The `async` feature enables `stream`.
- `AudioFileAnalyzer` is `Send`.
- Requires `doom-fish-utils` `>=0.4.1, <0.5`.
- `rust-version` is now 1.82 (was 1.76).

### Added

- `AsyncAudioFileAnalyzer::add_request_stream`.
- `LiveEvent`.

## [0.6.5] - 2026-06-06

- The async file analyzer copies the caller's audio path before starting its
  Task instead of reading the freed C string, and the Swift bridge gained
  leak fixes and ABI layout assertions.

## [0.6.4] - 2026-05-20

- Phase 32 completeness + async sweep.
- Added `AsyncAudioStreamAnalyzer`, `AudioStreamAnalysisStream`, and `AudioStreamAnalysisEvent` for bounded async streaming over `SNAudioStreamAnalyzer` observer callbacks.
- Refreshed the coverage docs against `MacOSX26.5.sdk`.

## [0.6.3] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.6.2] - 2026-05-18

- Widen doom-fish-utils version bound to `<0.3` so 0.2.x resolves.

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
