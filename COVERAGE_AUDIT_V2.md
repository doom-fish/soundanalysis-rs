# soundanalysis-rs coverage audit v2 (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 46
VERIFIED: 46
GAPS: 0
EXEMPT: 0
COVERAGE_PCT: 100.0

This audit enumerates all public macOS symbols in `SoundAnalysis.framework` (MacOSX26.2.sdk) across all headers: SNAnalyzer.h, SNAnalysis.h, SNClassificationResult.h, SNClassifySoundRequest.h, SNDefines.h, SNError.h, SNRequest.h, SNResult.h, SNTimeDurationConstraint.h, SNTypes.h. Every exported symbol is covered by the crate's safe Rust API (via `src/` public types and `swift-bridge/` glue code) or exposed via a type alias. The existing test suite (`tests/api_coverage.rs`) validates this coverage.

## 🟢 VERIFIED

| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| `SNRequest` | protocol | SNRequest.h | `SNRequest` trait, `AnalysisRequest` |
| `SNResult` | protocol | SNResult.h | `SNResult` trait, `AnalysisResult` |
| `SNResultsObserving` | protocol | SNResult.h | `ResultsObserver` trait, `SNResultsObserving` alias |
| `SNResultsObserving.request(_:didProduceResult:)` | protocol method | SNResult.h | `ResultsObserver::did_produce_result` |
| `SNResultsObserving.request(_:didFailWithError:)` | protocol method | SNResult.h | `ResultsObserver::did_fail_with_error` |
| `SNResultsObserving.requestDidComplete(_:)` | protocol method | SNResult.h | `ResultsObserver::did_complete` |
| `SNErrorDomain` | exported string constant | SNError.h | `error_domain()`, `SNErrorDomain` |
| `SNErrorCode` | error enum typedef | SNError.h | `ErrorCode`, `SNErrorCode` type alias |
| `SNErrorCodeUnknownError` | error value | SNError.h | `ErrorCode::UnknownError` |
| `SNErrorCodeOperationFailed` | error value | SNError.h | `ErrorCode::OperationFailed` |
| `SNErrorCodeInvalidFormat` | error value | SNError.h | `ErrorCode::InvalidFormat` |
| `SNErrorCodeInvalidModel` | error value | SNError.h | `ErrorCode::InvalidModel` |
| `SNErrorCodeInvalidFile` | error value | SNError.h | `ErrorCode::InvalidFile` |
| `SNClassifierIdentifier` | typed enum (NSString) | SNTypes.h | `ClassifierIdentifier`, `SNClassifierIdentifier` |
| `SNClassifierIdentifierVersion1` | exported constant | SNTypes.h | `ClassifierIdentifier::Version1`, `SNClassifierIdentifier::Version1` |
| `SNTimeDurationConstraintType` | enum typedef | SNTimeDurationConstraint.h | `TimeDurationConstraintType` |
| `SNTimeDurationConstraintTypeEnumerated` | enum value | SNTimeDurationConstraint.h | `TimeDurationConstraintType::Enumerated` |
| `SNTimeDurationConstraintTypeRange` | enum value | SNTimeDurationConstraint.h | `TimeDurationConstraintType::Range` |
| `SNTimeDurationConstraint` | interface | SNTimeDurationConstraint.h | `TimeDurationConstraint`, `SNTimeDurationConstraint` |
| `SNTimeDurationConstraint.type` | property (readonly) | SNTimeDurationConstraint.h | `TimeDurationConstraint::constraint_type()` |
| `SNTimeDurationConstraint.enumeratedDurations` | property (readonly) | SNTimeDurationConstraint.h | `TimeDurationConstraint::enumerated_durations()` |
| `SNTimeDurationConstraint.durationRange` | property (readonly) | SNTimeDurationConstraint.h | `TimeDurationConstraint::duration_range()` |
| `SNTimeDurationConstraint.initWithEnumeratedDurations(_:)` | designated initializer | SNTimeDurationConstraint.h | `TimeDurationConstraint::enumerated()` |
| `SNTimeDurationConstraint.initWithDurationRange(_:)` | designated initializer | SNTimeDurationConstraint.h | `TimeDurationConstraint::range()` |
| `SNAudioStreamAnalyzer` | interface | SNAnalyzer.h | `AudioStreamAnalyzer`, `SNAudioStreamAnalyzer` |
| `SNAudioStreamAnalyzer.initWithFormat(_:)` | designated initializer | SNAnalyzer.h | `AudioStreamAnalyzer::new()` |
| `SNAudioStreamAnalyzer.addRequest(_:withObserver:error:)` | instance method | SNAnalyzer.h | `AudioStreamAnalyzer::add_request()` |
| `SNAudioStreamAnalyzer.removeRequest(_:)` | instance method | SNAnalyzer.h | `AudioStreamAnalyzer::remove_request()` |
| `SNAudioStreamAnalyzer.removeAllRequests()` | instance method | SNAnalyzer.h | `AudioStreamAnalyzer::remove_all_requests()` |
| `SNAudioStreamAnalyzer.analyzeAudioBuffer(_:atAudioFramePosition:)` | instance method | SNAnalyzer.h | `AudioStreamAnalyzer::analyze_audio_buffer()` |
| `SNAudioStreamAnalyzer.completeAnalysis()` | instance method | SNAnalyzer.h | `AudioStreamAnalyzer::complete_analysis()` |
| `SNAudioFileAnalyzer` | interface | SNAnalyzer.h | `AudioFileAnalyzer`, `SNAudioFileAnalyzer` |
| `SNAudioFileAnalyzer.initWithURL(_:error:)` | designated initializer | SNAnalyzer.h | `AudioFileAnalyzer::new()` |
| `SNAudioFileAnalyzer.addRequest(_:withObserver:error:)` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::add_request()` |
| `SNAudioFileAnalyzer.removeRequest(_:)` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::remove_request()` |
| `SNAudioFileAnalyzer.removeAllRequests()` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::remove_all_requests()` |
| `SNAudioFileAnalyzer.analyze()` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::analyze()` |
| `SNAudioFileAnalyzer.analyzeWithCompletionHandler(_:)` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::analyze_with_completion_handler()` |
| `SNAudioFileAnalyzer.cancelAnalysis()` | instance method | SNAnalyzer.h | `AudioFileAnalyzer::cancel_analysis()` |
| `SNClassifySoundRequest` | interface | SNClassifySoundRequest.h | `ClassifySoundRequest`, `SNClassifySoundRequest` |
| `SNClassifySoundRequest.overlapFactor` | property (readwrite) | SNClassifySoundRequest.h | `ClassifySoundRequest::overlap_factor()`, `set_overlap_factor()` |
| `SNClassifySoundRequest.windowDuration` | property (readwrite) | SNClassifySoundRequest.h | `ClassifySoundRequest::window_duration()`, `set_window_duration()` |
| `SNClassifySoundRequest.windowDurationConstraint` | property (readonly) | SNClassifySoundRequest.h | `ClassifySoundRequest::window_duration_constraint()` |
| `SNClassifySoundRequest.knownClassifications` | property (readonly) | SNClassifySoundRequest.h | `ClassifySoundRequest::known_classifications()`, module-level `known_classifications()` |
| `SNClassifySoundRequest.initWithMLModel(_:error:)` | designated initializer | SNClassifySoundRequest.h | `ClassifySoundRequest::with_model_file()`, `classify_file_with_model()` |
| `SNClassifySoundRequest.initWithClassifierIdentifier(_:error:)` | designated initializer | SNClassifySoundRequest.h | `ClassifySoundRequest::with_classifier_identifier()`, `ClassifySoundRequest::version1()` |
| `SNClassification` | interface | SNClassificationResult.h | `Classification`, `SNClassification` |
| `SNClassification.identifier` | property (readonly) | SNClassificationResult.h | `Classification::identifier` |
| `SNClassification.confidence` | property (readonly) | SNClassificationResult.h | `Classification::confidence` |
| `SNClassificationResult` | interface | SNClassificationResult.h | `ClassificationResult`, `SNClassificationResult` |
| `SNClassificationResult.classifications` | property (readonly) | SNClassificationResult.h | `ClassificationResult::classifications` |
| `SNClassificationResult.timeRange` | property (readonly) | SNClassificationResult.h | `ClassificationResult::time_range()`, `TimeRange`, `SNTimeRange` |
| `SNClassificationResult.classificationForIdentifier(_:)` | instance method | SNClassificationResult.h | `ClassificationResult::classification_for_identifier()` |

## 🔴 GAPS

| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| _None_ | — | — | All public macOS `SoundAnalysis` symbols are wrapped. |

## ⏭️ EXEMPT

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| _None_ | — | — | — | — |
