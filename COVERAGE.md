# SoundAnalysis.framework coverage audit

Audited against the macOS `SoundAnalysis.framework` headers shipped in the Xcode 26.2 SDK (`MacOSX26.2.sdk`).

| Header | Apple API | Rust surface | Status | Notes |
| --- | --- | --- | --- | --- |
| `SNTypes.h` | `SNClassifierIdentifier` | `ClassifierIdentifier`, `SNClassifierIdentifier` | ✅ implemented | Apple’s typed enum maps to the safe Rust enum plus an Apple-style alias. |
| `SNTypes.h` | `SNClassifierIdentifierVersion1` | `ClassifierIdentifier::Version1`, `SNClassifierIdentifier::Version1` | ✅ implemented | Built-in classifier identifier is exposed directly. |
| Requested audit target | `SNClassifier` | `ClassifierIdentifier`, `SNClassifierIdentifier` | ✅ implemented | Current macOS headers expose `SNClassifierIdentifier`, not a separate `SNClassifier` type. |
| `SNRequest.h` | `SNRequest` | `SNRequest`, `AnalysisRequest` | ✅ implemented | Marker traits model the protocol surface without exposing raw Objective-C. |
| `SNResult.h` | `SNResult` | `SNResult`, `AnalysisResult` | ✅ implemented | Marker traits model the result protocol. |
| `SNResult.h` | `SNResultsObserving::request(_:didProduceResult:)` | `ResultsObserver::did_produce_result`, `SNResultsObserving` | ✅ implemented | Rust observers receive `ClassifySoundRequest` + `ClassificationResult`. |
| `SNResult.h` | `SNResultsObserving::request(_:didFailWithError:)` | `ResultsObserver::did_fail_with_error`, `SNResultsObserving` | ✅ implemented | Error callbacks surface as `SAError`. |
| `SNResult.h` | `SNResultsObserving::requestDidComplete(_:)` | `ResultsObserver::did_complete`, `SNResultsObserving` | ✅ implemented | Completion callback is bridged for file and stream analyzers. |
| `SNClassifySoundRequest.h` | `overlapFactor` | `ClassifySoundRequest::overlap_factor`, `set_overlap_factor` | ✅ implemented | Getter/setter validated in Rust before reaching Swift. |
| `SNClassifySoundRequest.h` | `windowDuration` | `ClassifySoundRequest::window_duration`, `set_window_duration` | ✅ implemented | Exposed as seconds for ergonomic Rust access. |
| `SNClassifySoundRequest.h` | `windowDurationConstraint` | `ClassifySoundRequest::window_duration_constraint` | ✅ implemented | Bridged through `TimeDurationConstraint` / `SNTimeDurationConstraint`. |
| `SNClassifySoundRequest.h` | `knownClassifications` | `ClassifySoundRequest::known_classifications`, `known_classifications()` | ✅ implemented | Available on the request and the crate-level convenience helper. |
| `SNClassifySoundRequest.h` | `initWithMLModel:error:` | `ClassifySoundRequest::with_model_file` | ✅ implemented | Loads a custom Core ML sound model through the Swift bridge. |
| `SNClassifySoundRequest.h` | `initWithClassifierIdentifier:error:` | `ClassifySoundRequest::with_classifier_identifier`, `version1` | ✅ implemented | Built-in classifier request creation is exposed directly. |
| Requested audit target | Custom CoreML model integration | `ClassifySoundRequest::with_model_file`, `classify_file_with_model` | ✅ implemented | Both the request surface and one-shot helper cover custom model loading. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.initWithURL(_:error:)` | `AudioFileAnalyzer::new`, `SNAudioFileAnalyzer` | ✅ implemented | Safe constructor validates UTF-8 paths and retains the analyzer box. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.addRequest(_:withObserver:error:)` | `AudioFileAnalyzer::add_request` | ✅ implemented | Observer callbacks are retained on the Rust side until removal/drop. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.removeRequest(_:)` | `AudioFileAnalyzer::remove_request` | ✅ implemented | Removes the request and drops the associated observer state. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.removeAllRequests()` | `AudioFileAnalyzer::remove_all_requests` | ✅ implemented | Clears all request/observer pairs. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.analyze()` | `AudioFileAnalyzer::analyze` | ✅ implemented | Synchronous analysis errors are surfaced as `SAError`. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.analyzeWithCompletionHandler(_:)` | `AudioFileAnalyzer::analyze_with_completion_handler` | ✅ implemented | Swift async completion is bridged back to sync Rust with `DispatchSemaphore`. |
| `SNAnalyzer.h` | `SNAudioFileAnalyzer.cancelAnalysis()` | `AudioFileAnalyzer::cancel_analysis` | ✅ implemented | Cancellation is exposed directly. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.initWithFormat(_:)` | `AudioStreamAnalyzer::new`, `SNAudioStreamAnalyzer` | ✅ implemented | Accepts validated PCM formats only. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.addRequest(_:withObserver:error:)` | `AudioStreamAnalyzer::add_request` | ✅ implemented | Observer lifecycle mirrors the file analyzer surface. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.removeRequest(_:)` | `AudioStreamAnalyzer::remove_request` | ✅ implemented | Removes a single live-stream request. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.removeAllRequests()` | `AudioStreamAnalyzer::remove_all_requests` | ✅ implemented | Clears all live-stream requests. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.analyzeAudioBuffer(_:atAudioFramePosition:)` | `AudioStreamAnalyzer::analyze_audio_buffer` | ✅ implemented | Interleaved and planar PCM layouts are converted to `AVAudioPCMBuffer` in Swift. |
| `SNAnalyzer.h` | `SNAudioStreamAnalyzer.completeAnalysis()` | `AudioStreamAnalyzer::complete_analysis` | ✅ implemented | Stream completion is exposed directly. |
| `SNTimeDurationConstraint.h` | `type` | `TimeDurationConstraint::constraint_type` | ✅ implemented | Matches Apple’s enumerated vs range discriminator. |
| `SNTimeDurationConstraint.h` | `enumeratedDurations` | `TimeDurationConstraint::enumerated_durations` | ✅ implemented | Exposed as seconds. |
| `SNTimeDurationConstraint.h` | `durationRange` | `TimeDurationConstraint::duration_range` | ✅ implemented | Exposed via `TimeRange` / `SNTimeRange`. |
| `SNTimeDurationConstraint.h` | `initWithEnumeratedDurations(_:)` | `TimeDurationConstraint::enumerated` | ✅ implemented | Constructed entirely through the Swift bridge. |
| `SNTimeDurationConstraint.h` | `initWithDurationRange(_:)` | `TimeDurationConstraint::range` | ✅ implemented | Constructed entirely through the Swift bridge. |
| Requested audit target | `SNTimeRange` | `TimeRange`, `SNTimeRange` | ✅ implemented | The framework uses `CMTimeRange`; the crate exposes a small Rust wrapper and Apple-style alias for those fields. |
| `SNClassificationResult.h` | `SNClassification.identifier` | `Classification::identifier`, `SNClassification` | ✅ implemented | Preserved as owned UTF-8 `String`. |
| `SNClassificationResult.h` | `SNClassification.confidence` | `Classification::confidence`, `SNClassification` | ✅ implemented | Preserved as `f64`. |
| `SNClassificationResult.h` | `SNClassificationResult.classifications` | `ClassificationResult::classifications`, `SNClassificationResult` | ✅ implemented | Ordered exactly as SoundAnalysis returns them. |
| `SNClassificationResult.h` | `SNClassificationResult.timeRange` | `ClassificationResult::{time_start,time_duration,time_range()}` | ✅ implemented | Time range is available both as flattened seconds and as the reusable `TimeRange` / `SNTimeRange` wrapper. |
| `SNClassificationResult.h` | `SNClassificationResult.classificationForIdentifier(_:)` | `ClassificationResult::classification_for_identifier` | ✅ implemented | Mirrors the framework helper. |
| Requested audit target | `SNDetectSoundEventRequest` | — | ⏭️ skipped | No public `SNDetectSoundEventRequest` symbol exists in the Xcode 26.2 SoundAnalysis headers or module interfaces. |
| Requested audit target | `SNAcousticFeaturePrintRequest` | — | ⏭️ skipped | No public `SNAcousticFeaturePrintRequest` symbol exists in the Xcode 26.2 SoundAnalysis headers or module interfaces. |
| Requested audit target | `SNAcousticFeaturePrint` | — | ⏭️ skipped | No public `SNAcousticFeaturePrint` symbol exists in the Xcode 26.2 SoundAnalysis headers or module interfaces. |
