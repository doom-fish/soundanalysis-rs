//! Tests for async API

#![cfg(all(test, feature = "async"))]

use soundanalysis::async_api::AsyncAudioFileAnalyzer;

#[test]
fn test_async_analyzer_creation() {
    // Test that we can create an analyzer even if the path doesn't exist
    // This just tests the creation logic, not actual analysis
    let result = AsyncAudioFileAnalyzer::new("/nonexistent/audio.mp3");
    assert!(result.is_ok(), "AsyncAudioFileAnalyzer::new should handle nonexistent paths");
}

#[test]
fn test_async_analyzer_invalid_path() {
    // Test with path containing NUL bytes
    let result = AsyncAudioFileAnalyzer::new("path\0with\0nul");
    assert!(result.is_err(), "Should reject paths with NUL bytes");
}

#[test]
fn test_async_analyzer_new_with_valid_path() {
    let analyzer = AsyncAudioFileAnalyzer::new("/tmp/test.wav");
    assert!(analyzer.is_ok());
}
