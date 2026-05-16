//! API-surface coverage harness for `soundanalysis`.

#![allow(clippy::cast_precision_loss, clippy::iter_on_single_items)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn sdk_root() -> PathBuf {
    let out = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-path"])
        .output()
        .expect("xcrun");
    assert!(out.status.success());
    PathBuf::from(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn collect_files(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extension, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

fn read_surface() -> String {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut swift_files = Vec::new();
    collect_files(
        &manifest.join("swift-bridge/Sources/SoundAnalysisBridge"),
        "swift",
        &mut swift_files,
    );
    swift_files.sort();

    let mut rust_files = Vec::new();
    collect_files(&manifest.join("src"), "rs", &mut rust_files);
    rust_files.sort();

    swift_files
        .into_iter()
        .chain(rust_files)
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_header(name: &str) -> String {
    read(&sdk_root().join(format!(
        "System/Library/Frameworks/SoundAnalysis.framework/Headers/{name}.h"
    )))
}

fn extract_interface(header: &str, type_name: &str) -> String {
    let needle = regex_lite::Regex::new(&format!(r"@interface\s+{type_name}\b")).unwrap();
    let Some(start) = needle.find(header) else {
        return String::new();
    };
    let rest = &header[start.start()..];
    let Some(end_off) = rest.find("@end") else {
        return rest.to_string();
    };
    rest[..end_off].to_string()
}

fn extract_protocol(header: &str, name: &str) -> String {
    let needle = regex_lite::Regex::new(&format!(r"@protocol\s+{name}\b")).unwrap();
    let Some(start) = needle.find(header) else {
        return String::new();
    };
    let rest = &header[start.start()..];
    let Some(end_off) = rest.find("@end") else {
        return rest.to_string();
    };
    rest[..end_off].to_string()
}

fn extract_member_surface(body: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let method_re =
        regex_lite::Regex::new(r"(?m)^\s*[+\-]\s*\([^\)]*\)\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    for capture in method_re.captures_iter(body) {
        out.insert(capture[1].to_string());
    }
    let prop_re = regex_lite::Regex::new(
        r"(?m)^\s*@property\s*(?:\([^\)]*\))?\s*[^;]*?\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:NS_|API_|;)",
    )
    .unwrap();
    for capture in prop_re.captures_iter(body) {
        out.insert(capture[1].to_string());
    }
    out
}

fn aliases() -> BTreeMap<&'static str, Vec<&'static str>> {
    BTreeMap::from([
        ("initWithURL", vec!["(url:", "AudioFileAnalyzer::new"]),
        ("initWithFormat", vec!["(format:", "AudioStreamAnalyzer::new"]),
        ("initWithMLModel", vec!["(mlModel:", "with_model_file("]),
        ("initWithClassifierIdentifier", vec!["(classifierIdentifier:", "with_classifier_identifier("]),
        ("addRequest", vec![".add(", "add_request("]),
        ("removeRequest", vec![".remove(", "remove_request("]),
        ("removeAllRequests", vec!["removeAllRequests(", "remove_all_requests("]),
        ("analyzeWithCompletionHandler", vec!["analyze_with_completion_handler(", "analyze {"]),
        ("cancelAnalysis", vec!["cancelAnalysis(", "cancel_analysis("]),
        ("analyzeAudioBuffer", vec!["atAudioFramePosition", "analyze_audio_buffer("]),
        ("classificationForIdentifier", vec!["classification_for_identifier("]),
        ("type", vec!["constraint_type(", ".enumeratedDurations(", ".durationRange("]),
        ("enumeratedDurations", vec!["enumerated_durations(", ".enumeratedDurations("]),
        ("durationRange", vec!["duration_range(", ".durationRange("]),
        ("initWithEnumeratedDurations", vec!["TimeDurationConstraint::enumerated(", ".enumeratedDurations("]),
        ("initWithDurationRange", vec!["TimeDurationConstraint::range(", ".durationRange("]),
        ("requestDidComplete", vec!["requestDidComplete(", "did_complete("]),
    ])
}

fn references_in_surface(symbols: &BTreeSet<String>) -> BTreeSet<String> {
    let surface = read_surface();
    let aliases = aliases();
    symbols
        .iter()
        .filter(|name| {
            let pattern = format!(r"\b{}\b", regex_lite::escape(name));
            if regex_lite::Regex::new(&pattern).unwrap().is_match(&surface) {
                return true;
            }
            aliases
                .get(name.as_str())
                .is_some_and(|forms| forms.iter().any(|form| surface.contains(form)))
        })
        .cloned()
        .collect()
}

fn report(name: &str, apple: &BTreeSet<String>, ours: &BTreeSet<String>, omitted: &BTreeSet<String>) {
    let wrapped: BTreeSet<&String> = apple.intersection(ours).collect();
    let missing: BTreeSet<&String> = apple
        .difference(ours)
        .filter(|symbol| !omitted.contains(*symbol))
        .collect();
    let coverable = wrapped.len() + missing.len();
    let pct = if coverable == 0 {
        100.0
    } else {
        wrapped.len() as f64 / coverable as f64 * 100.0
    };
    println!(
        "\n=== {name} ===\n  apple={}, omitted={}, coverable={coverable}, wrapped={}, missing={}, pct={pct:.1}%",
        apple.len(),
        omitted.len(),
        wrapped.len(),
        missing.len(),
    );
    if !missing.is_empty() {
        for symbol in &missing {
            println!("  - {symbol}");
        }
    }
    assert!(pct >= 100.0, "{name}: {pct:.1}%");
}

fn omitted_set<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(String::from).collect()
}

#[test]
fn sn_classify_sound_request_coverage() {
    let header = read_header("SNClassifySoundRequest");
    let body = extract_interface(&header, "SNClassifySoundRequest");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set(["new"]);
    report("SNClassifySoundRequest", &apple, &ours, &omitted);
}

#[test]
fn sn_audio_file_analyzer_coverage() {
    let header = read_header("SNAnalyzer");
    let body = extract_interface(&header, "SNAudioFileAnalyzer");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set([]);
    report("SNAudioFileAnalyzer", &apple, &ours, &omitted);
}

#[test]
fn sn_audio_stream_analyzer_coverage() {
    let header = read_header("SNAnalyzer");
    let body = extract_interface(&header, "SNAudioStreamAnalyzer");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set([]);
    report("SNAudioStreamAnalyzer", &apple, &ours, &omitted);
}

#[test]
fn sn_classification_result_coverage() {
    let header = read_header("SNClassificationResult");
    let body = extract_interface(&header, "SNClassificationResult");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set(["new"]);
    report("SNClassificationResult", &apple, &ours, &omitted);
}

#[test]
fn sn_classification_coverage() {
    let header = read_header("SNClassificationResult");
    let body = extract_interface(&header, "SNClassification");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set(["new"]);
    report("SNClassification", &apple, &ours, &omitted);
}

#[test]
fn sn_time_duration_constraint_coverage() {
    let header = read_header("SNTimeDurationConstraint");
    let body = extract_interface(&header, "SNTimeDurationConstraint");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set(["new"]);
    report("SNTimeDurationConstraint", &apple, &ours, &omitted);
}

#[test]
fn sn_results_observing_coverage() {
    let header = read_header("SNResult");
    let body = extract_protocol(&header, "SNResultsObserving");
    let apple = extract_member_surface(&body);
    let ours = references_in_surface(&apple);
    let omitted = omitted_set([]);
    report("SNResultsObserving", &apple, &ours, &omitted);
}
