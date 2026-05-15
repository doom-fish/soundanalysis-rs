//! API-surface coverage harness for `soundanalysis`.
//!
//! `SoundAnalysis` is an Obj-C framework with proper headers under
//! `SoundAnalysis.framework/Headers/`. Mirrors the family pattern
//! (header-based, Obj-C `@interface`).

#![allow(clippy::cast_precision_loss, clippy::iter_on_single_items)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

fn sdk_root() -> PathBuf {
    let out = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-path"])
        .output()
        .expect("xcrun");
    assert!(out.status.success());
    PathBuf::from(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

fn read(path: &PathBuf) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_bridge() -> String {
    read(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "swift-bridge/Sources/SoundAnalysisBridge/SoundAnalysis.swift",
    ))
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
    for c in method_re.captures_iter(body) {
        out.insert(c[1].to_string());
    }
    let prop_re = regex_lite::Regex::new(
        r"(?m)^\s*@property\s*(?:\([^\)]*\))?\s*[^;]*?\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:NS_|API_|;)",
    )
    .unwrap();
    for c in prop_re.captures_iter(body) {
        out.insert(c[1].to_string());
    }
    out
}

fn references_in_bridge(symbols: &BTreeSet<String>) -> BTreeSet<String> {
    let bridge = read_bridge();
    let aliases = swift_aliases();
    symbols
        .iter()
        .filter(|name| {
            let pattern = format!(r"\b{}\b", regex_lite::escape(name));
            if regex_lite::Regex::new(&pattern).unwrap().is_match(&bridge) {
                return true;
            }
            if let Some(form) = aliases.get(name.as_str()) {
                return bridge.contains(form);
            }
            false
        })
        .cloned()
        .collect()
}

fn swift_aliases() -> std::collections::BTreeMap<&'static str, &'static str> {
    [
        ("initWithURL", "(url:"),
        ("initWithFormat", "(format:"),
        ("initWithMLModel", "(mlModel:"),
        ("initWithClassifierIdentifier", "(classifierIdentifier:"),
        ("addRequest", "analyzer.add("),
        ("removeRequest", "removeRequest("),
        ("removeAllRequests", "removeAllRequests("),
    ]
    .into_iter()
    .collect()
}

fn report(name: &str, apple: &BTreeSet<String>, ours: &BTreeSet<String>, omitted: &BTreeSet<String>) {
    let wrapped: BTreeSet<&String> = apple.intersection(ours).collect();
    let missing: BTreeSet<&String> = apple
        .difference(ours)
        .filter(|s| !omitted.contains(*s))
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
        for s in &missing {
            println!("  - {s}");
        }
    }
    assert!(pct >= 100.0, "{name}: {pct:.1}%");
}

fn omitted_set<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(String::from).collect()
}

// ---- Tests ----

#[test]
fn sn_classify_sound_request_coverage() {
    let header = read_header("SNClassifySoundRequest");
    let body = extract_interface(&header, "SNClassifySoundRequest");
    let apple = extract_member_surface(&body);
    let ours = references_in_bridge(&apple);
    let omitted = omitted_set([
        // Per-request tunables — v0.2 builder will surface them.
        "overlapFactor",
        "windowDuration",
        "windowDurationConstraint",
        // Custom-MLModel attachment — v0.2.
        "initWithMLModel",
        // Bridge uses the .version1 enum case directly via Swift, not via
        // the literal Obj-C selector text.
        "initWithClassifierIdentifier",
        // `+ new` is `NS_UNAVAILABLE` on every SoundAnalysis class.
        "new",
    ]);
    report("SNClassifySoundRequest", &apple, &ours, &omitted);
}

#[test]
fn sn_audio_file_analyzer_coverage() {
    let header = read_header("SNAnalyzer");
    let body = extract_interface(&header, "SNAudioFileAnalyzer");
    let apple = extract_member_surface(&body);
    let ours = references_in_bridge(&apple);
    let omitted = omitted_set([
        // Removal API not used in v0.1 (one-shot analysis per call).
        "removeRequest",
        "removeAllRequests",
        // Async + cancellable variants — v0.2 (current bridge uses the
        // synchronous `analyze()` form).
        "analyzeWithCompletionHandler",
        "cancelAnalysis",
    ]);
    report("SNAudioFileAnalyzer", &apple, &ours, &omitted);
}

#[test]
fn sn_classification_result_coverage() {
    let header = read_header("SNClassificationResult");
    let body = extract_interface(&header, "SNClassificationResult");
    let apple = extract_member_surface(&body);
    let ours = references_in_bridge(&apple);
    let omitted = omitted_set([
        // Per-identifier random-access lookup; v0.1 returns the eager Vec
        // of all classifications and lets Rust callers filter/find.
        "classificationForIdentifier",
        "new",
    ]);
    report("SNClassificationResult", &apple, &ours, &omitted);
}

#[test]
fn sn_classification_coverage() {
    let header = read_header("SNClassificationResult");
    let body = extract_interface(&header, "SNClassification");
    let apple = extract_member_surface(&body);
    let ours = references_in_bridge(&apple);
    let omitted = omitted_set(["new"]);
    report("SNClassification", &apple, &ours, &omitted);
}

#[test]
fn sn_results_observing_coverage() {
    // Protocol — verify our CollectingObserver implements the full
    // delegate surface the analyzer expects.
    let header = read_header("SNResult");
    let body = extract_protocol(&header, "SNResultsObserving");
    let apple = extract_member_surface(&body);
    let ours = references_in_bridge(&apple);
    let omitted = omitted_set([
        // Bridge swift_aliases() handles the renamed selector
        // `request:didProduceResult:` -> `request(_:didProduce:)`.
        "request",
    ]);
    report("SNResultsObserving", &apple, &ours, &omitted);
}
