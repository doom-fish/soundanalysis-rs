use soundanalysis::{ClassifySoundRequest, SAError, TimeDurationConstraint};

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1.0e-9, "{left} != {right}");
}

#[test]
fn version1_request_exposes_labels_and_tuning_surface() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = ClassifySoundRequest::version1()?;
    request.set_overlap_factor(0.25)?;
    assert_close(request.overlap_factor(), 0.25);

    let window_duration = request.window_duration();
    assert!(window_duration.is_finite() && window_duration > 0.0);
    request.set_window_duration(window_duration)?;
    assert_close(request.window_duration(), window_duration);

    match request.window_duration_constraint()? {
        TimeDurationConstraint::Enumerated(durations) => {
            assert!(!durations.is_empty());
            assert!(durations
                .iter()
                .all(|duration| duration.is_finite() && *duration > 0.0));
        }
        TimeDurationConstraint::Range(range) => {
            assert!(range.start_seconds.is_finite());
            assert!(range.duration_seconds.is_finite());
            assert!(range.duration_seconds >= 0.0);
        }
    }

    let labels = request.known_classifications()?;
    assert!(
        labels.len() > 100,
        "expected many labels, got {}",
        labels.len()
    );
    assert!(labels.iter().any(|label| label == "speech"));
    assert!(labels.iter().any(|label| label == "music"));

    let cloned = request.clone();
    assert_close(cloned.overlap_factor(), 0.25);
    assert_close(cloned.window_duration(), window_duration);
    Ok(())
}

#[test]
fn request_validates_invalid_tuning_inputs_before_touching_framework() {
    let mut request = ClassifySoundRequest::version1().expect("version1 request");

    let overlap_error = request.set_overlap_factor(1.0).unwrap_err();
    assert!(
        matches!(overlap_error, SAError::InvalidArgument(message) if message.contains("overlap_factor"))
    );

    let window_error = request.set_window_duration(0.0).unwrap_err();
    assert!(
        matches!(window_error, SAError::InvalidArgument(message) if message.contains("window duration"))
    );
}
