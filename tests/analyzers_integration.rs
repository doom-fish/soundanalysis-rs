#![cfg(feature = "analyze_file")]

mod common;

use std::sync::{Arc, Mutex};

use soundanalysis::{AudioFileAnalyzer, ClassifySoundRequest, ResultsObserverFns, SAError};

#[test]
fn audio_file_analyzer_reports_results_and_rejects_duplicate_requests(
) -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech(
        "analyzers-file",
        "hello from the sound analysis file analyzer integration test",
    );

    let mut analyzer = AudioFileAnalyzer::new(&audio)?;
    let request = ClassifySoundRequest::version1()?;
    let observed = Arc::new(Mutex::new(Vec::new()));

    analyzer.add_request(
        &request,
        ResultsObserverFns::new({
            let observed = Arc::clone(&observed);
            move |_request, result| {
                observed.lock().expect("observed results").push(result);
            }
        }),
    )?;

    let duplicate = analyzer
        .add_request(&request, ResultsObserverFns::new(|_, _| {}))
        .unwrap_err();
    assert!(
        matches!(duplicate, SAError::InvalidArgument(message) if message.contains("already added"))
    );

    let reached_end = analyzer.analyze_with_completion_handler()?;
    assert!(
        reached_end,
        "expected analyzer to reach end of synthesized speech file"
    );

    let observed = observed.lock().expect("observed results");
    assert!(
        !observed.is_empty(),
        "expected at least one analyzer result"
    );
    assert!(observed.iter().all(|result| result.time_duration > 0.0));

    let top = observed
        .iter()
        .find_map(|result| result.top())
        .expect("at least one top classification");
    assert!((0.0..=1.0).contains(&top.confidence));
    drop(observed);

    analyzer.remove_request(&request);
    analyzer.remove_all_requests();
    Ok(())
}
