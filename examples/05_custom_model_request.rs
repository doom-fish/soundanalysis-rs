//! Exercise the custom Core ML request surface without requiring a bundled model.
//!
//! Run: `cargo run --example 05_custom_model_request`

use std::path::PathBuf;
use std::process::Command;

use soundanalysis::{classify_file_with_model, ClassifySoundRequest, SAError};

fn synthesize_speech() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts");
    std::fs::create_dir_all(&artifacts)?;
    let audio = artifacts.join("soundanalysis-custom-model.aiff");
    let _ = std::fs::remove_file(&audio);

    let status = Command::new("/usr/bin/say")
        .args([
            "-o",
            audio.to_str().unwrap(),
            "custom model integration smoke test",
        ])
        .status()?;
    if !status.success() {
        return Err(format!("say failed: {status}").into());
    }
    Ok(audio)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio = synthesize_speech()?;
    let missing_model =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/example-artifacts/missing.mlmodelc");

    match ClassifySoundRequest::with_model_file(&missing_model) {
        Err(SAError::RequestCreateFailed(message)) => {
            println!("request creation failed as expected: {message}");
        }
        Err(other) => return Err(format!("unexpected request error: {other}").into()),
        Ok(_) => return Err("missing model unexpectedly loaded".into()),
    }

    match classify_file_with_model(&audio, &missing_model) {
        Err(SAError::RequestCreateFailed(message)) => {
            println!("file classification failed as expected: {message}");
        }
        Err(other) => {
            return Err(format!("unexpected classify_file_with_model error: {other}").into())
        }
        Ok(_) => return Err("missing custom model unexpectedly classified audio".into()),
    }

    Ok(())
}
