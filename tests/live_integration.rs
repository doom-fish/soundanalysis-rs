#![cfg(feature = "stream")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use soundanalysis::{
    start_live_classification, Classification, LiveClassification, LiveEvent, SAError, StreamUpdate,
};

extern "C" {
    fn sa_live_microphone_authorized() -> bool;
}

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

    let event = LiveEvent::Update(update);
    assert_eq!(event.clone(), event);
    assert_ne!(event, LiveEvent::Completed);
}

#[test]
fn live_classification_reports_completion_when_stopped() {
    if !unsafe { sa_live_microphone_authorized() } {
        eprintln!("skipping: microphone access is not authorized, and this test never prompts");
        return;
    }

    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);
    match start_live_classification(move |event| {
        sink.lock().expect("events").push(event);
    }) {
        Ok(session) => {
            std::thread::sleep(Duration::from_millis(300));
            drop(session);
            let received = events.lock().expect("events").clone();
            assert!(
                received.contains(&LiveEvent::Completed),
                "expected a completion event when the session stops: {received:?}"
            );
        }
        Err(SAError::AnalysisFailed(message)) => {
            assert!(
                !message.trim().is_empty(),
                "expected an explanatory live-analysis error"
            );
        }
        Err(other) => panic!("unexpected live classification error: {other}"),
    }
    assert_eq!(Arc::strong_count(&events), 1);
}
