#![cfg(feature = "analyze_file")]

mod common;

use soundanalysis::{classify_file, known_classifications};

#[test]
#[allow(clippy::float_cmp)]
fn classify_file_returns_ranked_results_for_generated_speech(
) -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech(
        "classification-generated-speech",
        "this is a soundanalysis integration test that generates a spoken utterance long enough to classify as speech",
    );

    let results = classify_file(&audio)?;
    assert!(
        !results.is_empty(),
        "expected at least one classification window"
    );

    let known = known_classifications()?;
    assert!(
        known.len() > 100,
        "expected many known labels, got {}",
        known.len()
    );

    let (result, top) = results
        .iter()
        .find_map(|result| result.top().map(|top| (result, top)))
        .expect("at least one top classification");

    assert!(result.time_start >= 0.0);
    assert!(result.time_duration > 0.0);
    assert!((0.0..=1.0).contains(&top.confidence));
    assert_eq!(
        result.classification_for_identifier(&top.identifier),
        Some(top)
    );

    let time_range = result.time_range();
    assert_eq!(time_range.start_seconds, result.time_start);
    assert_eq!(time_range.duration_seconds, result.time_duration);
    assert!(known.iter().any(|label| label == &top.identifier));
    Ok(())
}
