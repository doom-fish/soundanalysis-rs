//! Quick smoke test for the Apple-style alias surface.
//!
//! Run: `cargo run --example 04_apple_aliases`

use soundanalysis::{
    error_domain, ErrorCode, SNClassification, SNClassificationResult, SNClassifierIdentifier,
    SNClassifySoundRequest, SNErrorCode, SNErrorDomain, SNRequest, SNResult,
    SNTimeDurationConstraint, SNTimeRange, TimeDurationConstraint, TimeRange,
};

const fn assert_request<T: SNRequest>() {}
const fn assert_result<T: SNResult>() {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_request::<SNClassifySoundRequest>();
    assert_result::<SNClassificationResult>();

    let request =
        SNClassifySoundRequest::with_classifier_identifier(SNClassifierIdentifier::Version1)?;
    let error_code: SNErrorCode = ErrorCode::InvalidModel;
    assert_eq!(SNErrorDomain(), error_domain());

    let range: SNTimeRange = TimeRange {
        start_seconds: 0.5,
        duration_seconds: 1.5,
    };
    let constraint: SNTimeDurationConstraint =
        TimeDurationConstraint::range(range.start_seconds, range.duration_seconds)?;
    let classification = SNClassification {
        identifier: "speech".into(),
        confidence: 0.9,
    };
    let result = SNClassificationResult {
        time_start: range.start_seconds,
        time_duration: range.duration_seconds,
        classifications: vec![classification],
    };

    println!(
        "alias request overlap={:.2}, error_domain={}, invalid_model_code={}, constraint={:?}, time_range={:?}, top={:?}",
        request.overlap_factor(),
        SNErrorDomain(),
        error_code.as_raw(),
        constraint.constraint_type(),
        result.time_range(),
        result
            .top()
            .map(|value| (&value.identifier, value.confidence))
    );
    Ok(())
}
