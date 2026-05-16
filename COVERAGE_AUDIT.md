# soundanalysis-rs coverage audit (vs MacOSX26.2.sdk)

Audited against `SoundAnalysis.framework` in Xcode 26.2 (`MacOSX26.2.sdk`).

This audit counts top-level exported SoundAnalysis types/constants/protocols **and** the public Objective-C properties/methods on those interfaces/protocols, excluding `NS_UNAVAILABLE` initializers. The crate’s existing `tests/api_coverage.rs` already verifies the request/result/analyzer member surface; this document extends that audit to the full framework surface, including exported error symbols.

SDK_PUBLIC_SYMBOLS: 46
VERIFIED: 44
GAPS: 2
EXEMPT: 0
COVERAGE_PCT: 95.7%

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| `SNClassifierIdentifier` | typed enum | `SNTypes.h` | `ClassifierIdentifier`, `SNClassifierIdentifier` |
| `SNClassifierIdentifierVersion1` | exported constant | `SNTypes.h` | `ClassifierIdentifier::Version1`, `SNClassifierIdentifier::Version1`, `ClassifySoundRequest::version1` |
| `SNRequest` | protocol | `SNRequest.h` | `SNRequest`, `AnalysisRequest` |
| `SNResult` | protocol | `SNResult.h` | `SNResult`, `AnalysisResult` |
| `SNResultsObserving` | protocol | `SNResult.h` | `ResultsObserver`, `SNResultsObserving`, `ResultsObserverFns` |
| `SNResultsObserving.request(_:didProduceResult:)` | protocol method | `SNResult.h` | `ResultsObserver::did_produce_result` |
| `SNResultsObserving.request(_:didFailWithError:)` | protocol method | `SNResult.h` | `ResultsObserver::did_fail_with_error` |
| `SNResultsObserving.requestDidComplete(_:)` | protocol method | `SNResult.h` | `ResultsObserver::did_complete` |
| `SNAudioStreamAnalyzer` | interface | `SNAnalyzer.h` | `AudioStreamAnalyzer`, `SNAudioStreamAnalyzer` |
| `SNAudioStreamAnalyzer.initWithFormat(_:)` | initializer | `SNAnalyzer.h` | `AudioStreamAnalyzer::new` |
| `SNAudioStreamAnalyzer.addRequest(_:withObserver:error:)` | method | `SNAnalyzer.h` | `AudioStreamAnalyzer::add_request` |
| `SNAudioStreamAnalyzer.removeRequest(_:)` | method | `SNAnalyzer.h` | `AudioStreamAnalyzer::remove_request` |
| `SNAudioStreamAnalyzer.removeAllRequests()` | method | `SNAnalyzer.h` | `AudioStreamAnalyzer::remove_all_requests` |
| `SNAudioStreamAnalyzer.analyzeAudioBuffer(_:atAudioFramePosition:)` | method | `SNAnalyzer.h` | `AudioStreamAnalyzer::analyze_audio_buffer` |
| `SNAudioStreamAnalyzer.completeAnalysis()` | method | `SNAnalyzer.h` | `AudioStreamAnalyzer::complete_analysis` |
| `SNAudioFileAnalyzer` | interface | `SNAnalyzer.h` | `AudioFileAnalyzer`, `SNAudioFileAnalyzer` |
| `SNAudioFileAnalyzer.initWithURL(_:error:)` | initializer | `SNAnalyzer.h` | `AudioFileAnalyzer::new` |
| `SNAudioFileAnalyzer.addRequest(_:withObserver:error:)` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::add_request` |
| `SNAudioFileAnalyzer.removeRequest(_:)` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::remove_request` |
| `SNAudioFileAnalyzer.removeAllRequests()` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::remove_all_requests` |
| `SNAudioFileAnalyzer.analyze()` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::analyze` |
| `SNAudioFileAnalyzer.analyzeWithCompletionHandler(_:)` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::analyze_with_completion_handler` |
| `SNAudioFileAnalyzer.cancelAnalysis()` | method | `SNAnalyzer.h` | `AudioFileAnalyzer::cancel_analysis` |
| `SNTimeDurationConstraintType` | enum typedef | `SNTimeDurationConstraint.h` | `TimeDurationConstraintType` |
| `SNTimeDurationConstraint` | interface | `SNTimeDurationConstraint.h` | `TimeDurationConstraint`, `SNTimeDurationConstraint` |
| `SNTimeDurationConstraint.type` | property | `SNTimeDurationConstraint.h` | `TimeDurationConstraint::constraint_type` |
| `SNTimeDurationConstraint.enumeratedDurations` | property | `SNTimeDurationConstraint.h` | `TimeDurationConstraint::enumerated_durations` |
| `SNTimeDurationConstraint.durationRange` | property | `SNTimeDurationConstraint.h` | `TimeDurationConstraint::duration_range` |
| `SNTimeDurationConstraint.initWithEnumeratedDurations(_:)` | initializer | `SNTimeDurationConstraint.h` | `TimeDurationConstraint::enumerated` |
| `SNTimeDurationConstraint.initWithDurationRange(_:)` | initializer | `SNTimeDurationConstraint.h` | `TimeDurationConstraint::range` |
| `SNClassifySoundRequest` | interface | `SNClassifySoundRequest.h` | `ClassifySoundRequest`, `SNClassifySoundRequest` |
| `SNClassifySoundRequest.overlapFactor` | property | `SNClassifySoundRequest.h` | `ClassifySoundRequest::overlap_factor`, `ClassifySoundRequest::set_overlap_factor` |
| `SNClassifySoundRequest.windowDuration` | property | `SNClassifySoundRequest.h` | `ClassifySoundRequest::window_duration`, `ClassifySoundRequest::set_window_duration` |
| `SNClassifySoundRequest.windowDurationConstraint` | property | `SNClassifySoundRequest.h` | `ClassifySoundRequest::window_duration_constraint` |
| `SNClassifySoundRequest.knownClassifications` | property | `SNClassifySoundRequest.h` | `ClassifySoundRequest::known_classifications`, `known_classifications()` |
| `SNClassifySoundRequest.initWithMLModel(_:error:)` | initializer | `SNClassifySoundRequest.h` | `ClassifySoundRequest::with_model_file`, `classify_file_with_model` |
| `SNClassifySoundRequest.initWithClassifierIdentifier(_:error:)` | initializer | `SNClassifySoundRequest.h` | `ClassifySoundRequest::with_classifier_identifier`, `ClassifySoundRequest::version1` |
| `SNClassification` | interface | `SNClassificationResult.h` | `Classification`, `SNClassification` |
| `SNClassification.identifier` | property | `SNClassificationResult.h` | `Classification::identifier` |
| `SNClassification.confidence` | property | `SNClassificationResult.h` | `Classification::confidence` |
| `SNClassificationResult` | interface | `SNClassificationResult.h` | `ClassificationResult`, `SNClassificationResult` |
| `SNClassificationResult.classifications` | property | `SNClassificationResult.h` | `ClassificationResult::classifications` |
| `SNClassificationResult.timeRange` | property | `SNClassificationResult.h` | `ClassificationResult::{time_start, time_duration, time_range()}`, `TimeRange`, `SNTimeRange` |
| `SNClassificationResult.classificationForIdentifier(_:)` | method | `SNClassificationResult.h` | `ClassificationResult::classification_for_identifier` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| `SNErrorDomain` | exported constant | `SNError.h` | Public Rust errors are normalized into `SAError`; the crate does not expose Apple’s `SNErrorDomain` constant. |
| `SNErrorCode` | error enum typedef | `SNError.h` | `SAError` abstracts bridge failures into coarse variants and custom bridge status codes, so the framework’s raw `SNErrorCode` enum/cases are not publicly wrapped. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

Validation: `cargo test --quiet`
