#![cfg(feature = "stream")]

use soundanalysis::{
    start_live_classification, Classification, LiveClassification, SAError, StreamUpdate,
};

#[allow(clippy::missing_const_for_fn)]
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn live_api_types_are_send_sync_and_cloneable() {
    assert_send_sync::<LiveClassification>();

    let update = StreamUpdate {
        time_start: 1.5,
        time_duration: 0.5,
        classifications: vec![Classification {
            identifier: "speech".into(),
            confidence: 0.9,
        }],
    };

    assert_eq!(update.clone(), update);
    assert_eq!(update.classifications[0].identifier, "speech");
}

#[test]
fn live_classification_starts_or_reports_analysis_failure() {
    match start_live_classification(|_update| {}) {
        Ok(session) => drop(session),
        Err(SAError::AnalysisFailed(message)) => {
            assert!(
                !message.trim().is_empty(),
                "expected an explanatory live-analysis error"
            );
        }
        Err(other) => panic!("unexpected live classification error: {other}"),
    }
}
