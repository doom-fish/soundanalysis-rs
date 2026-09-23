#![cfg(all(test, feature = "async"))]

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use soundanalysis::async_api::{
    AnalyzeFileFuture, AsyncAudioFileAnalyzer, AudioStreamAnalysisEvent,
};
use soundanalysis::{ClassifySoundRequest, ResultsObserverFns, SAError};

#[allow(clippy::missing_const_for_fn)]
fn assert_send<T: Send>() {}

fn short_window_request() -> Result<ClassifySoundRequest, SAError> {
    let mut request = ClassifySoundRequest::version1()?;
    request.set_window_duration(1.0)?;
    Ok(request)
}

#[test]
fn async_analyzer_and_its_future_are_send() {
    assert_send::<AsyncAudioFileAnalyzer>();
    assert_send::<AnalyzeFileFuture<'static>>();
}

#[test]
fn async_analyzer_rejects_missing_files() {
    let result = AsyncAudioFileAnalyzer::new("/nonexistent/audio.mp3");
    assert!(
        matches!(result, Err(SAError::AudioLoadFailed(_))),
        "{result:?}"
    );
}

#[test]
fn async_analyzer_rejects_paths_with_nul_bytes() {
    let result = AsyncAudioFileAnalyzer::new("path\0with\0nul");
    assert!(
        matches!(result, Err(SAError::InvalidArgument(_))),
        "{result:?}"
    );
}

#[test]
fn async_analysis_delivers_results_and_completion() -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech("async-stream", "the quick brown fox jumps over the dog");
    let mut analyzer = AsyncAudioFileAnalyzer::new(&audio)?;
    let request = short_window_request()?;
    let events = analyzer.add_request_stream(&request, 1024)?;

    let reached_end = pollster::block_on(analyzer.analyze())?;
    assert!(reached_end);

    let mut results = 0;
    let mut completed = false;
    while let Some(event) = events.try_next() {
        match event {
            AudioStreamAnalysisEvent::Result(result) => {
                assert!(result.time_duration > 0.0);
                results += 1;
            }
            AudioStreamAnalysisEvent::Complete => completed = true,
            AudioStreamAnalysisEvent::Error(error) => panic!("unexpected analysis error: {error}"),
        }
    }
    assert!(results > 0, "expected classification results");
    assert!(completed, "expected a completion event");
    Ok(())
}

#[test]
fn async_analysis_feeds_callback_observers() -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech("async-callback", "sound analysis async observer");
    let mut analyzer = AsyncAudioFileAnalyzer::new(&audio)?;
    let request = short_window_request()?;
    let results = Arc::new(AtomicUsize::new(0));
    let completions = Arc::new(AtomicUsize::new(0));
    analyzer.add_request(
        &request,
        ResultsObserverFns::new({
            let results = Arc::clone(&results);
            move |_, _| {
                results.fetch_add(1, Ordering::SeqCst);
            }
        })
        .on_complete({
            let completions = Arc::clone(&completions);
            move |_| {
                completions.fetch_add(1, Ordering::SeqCst);
            }
        }),
    )?;

    assert!(pollster::block_on(analyzer.analyze())?);
    assert!(results.load(Ordering::SeqCst) > 0);
    assert_eq!(completions.load(Ordering::SeqCst), 1);

    analyzer.remove_all_requests();
    assert!(pollster::block_on(analyzer.analyze())?);
    assert_eq!(completions.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
fn dropping_an_analysis_future_cancels_the_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech(
        "async-cancel",
        "cancelling an asynchronous sound analysis should stop it before the end of the file",
    );
    let mut analyzer = AsyncAudioFileAnalyzer::new(&audio)?;
    let request = short_window_request()?;
    analyzer.add_request(&request, ResultsObserverFns::new(|_, _| {}))?;

    drop(analyzer.analyze());

    let deadline = Instant::now() + Duration::from_secs(30);
    let reached_end = loop {
        match pollster::block_on(analyzer.analyze()) {
            Err(SAError::AnalysisFailed(message))
                if message.contains("in progress") && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            other => break other?,
        }
    };
    assert!(
        !reached_end,
        "a cancelled SNAudioFileAnalyzer must not report reaching the end"
    );
    Ok(())
}

#[test]
fn zero_capacity_streams_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let audio = common::synthesize_speech("async-capacity", "capacity");
    let mut analyzer = AsyncAudioFileAnalyzer::new(&audio)?;
    let request = short_window_request()?;
    assert!(matches!(
        analyzer.add_request_stream(&request, 0),
        Err(SAError::InvalidArgument(_))
    ));
    Ok(())
}
