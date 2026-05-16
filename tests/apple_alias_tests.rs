use std::path::PathBuf;

use soundanalysis::{
    AnalysisRequest, AnalysisResult, Classification, ClassificationResult, ClassifierIdentifier,
    ClassifySoundRequest, SAError, SNClassification, SNClassificationResult,
    SNClassifierIdentifier, SNClassifySoundRequest, SNRequest, SNResult, SNTimeDurationConstraint,
    SNTimeRange, TimeDurationConstraint, TimeRange,
};

const fn assert_request<T: SNRequest + AnalysisRequest>() {}
const fn assert_result<T: SNResult + AnalysisResult>() {}

#[test]
fn apple_aliases_typecheck_and_match_safe_wrappers() {
    assert_request::<ClassifySoundRequest>();
    assert_request::<SNClassifySoundRequest>();
    assert_result::<ClassificationResult>();
    assert_result::<SNClassificationResult>();

    let classifier: SNClassifierIdentifier = ClassifierIdentifier::Version1;
    assert_eq!(classifier, SNClassifierIdentifier::Version1);

    let range: SNTimeRange = TimeRange {
        start_seconds: 0.25,
        duration_seconds: 1.5,
    };
    let constraint: SNTimeDurationConstraint =
        TimeDurationConstraint::range(range.start_seconds, range.duration_seconds).unwrap();
    assert_eq!(constraint.duration_range(), Some(range));

    let classification: SNClassification = Classification {
        identifier: "speech".into(),
        confidence: 0.9,
    };
    let result: SNClassificationResult = ClassificationResult {
        time_start: 0.0,
        time_duration: 1.5,
        classifications: vec![classification.clone()],
    };
    assert_eq!(
        result.time_range(),
        TimeRange {
            start_seconds: 0.0,
            duration_seconds: 1.5,
        }
    );
    assert_eq!(
        result.classification_for_identifier("speech"),
        Some(&classification)
    );
}

#[test]
fn missing_custom_model_surfaces_request_create_failed() {
    let missing_model =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts/missing.mlmodelc");
    let error = ClassifySoundRequest::with_model_file(missing_model).unwrap_err();
    assert!(matches!(error, SAError::RequestCreateFailed(_)));
}
