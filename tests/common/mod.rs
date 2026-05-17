use std::path::PathBuf;
use std::process::Command;

pub fn synthesize_speech(stem: &str, text: &str) -> PathBuf {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-artifacts");
    std::fs::create_dir_all(&artifacts).expect("create target/test-artifacts");

    let audio = artifacts.join(format!("{stem}.aiff"));
    let _ = std::fs::remove_file(&audio);

    let status = Command::new("/usr/bin/say")
        .args(["-o", audio.to_str().expect("utf8 audio path"), text])
        .status()
        .expect("invoke /usr/bin/say");
    assert!(status.success(), "say failed: {status}");

    let metadata = std::fs::metadata(&audio).expect("speech artifact metadata");
    assert!(metadata.len() > 0, "speech artifact is empty");
    audio
}
