use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn create_writes_json_and_markdown_packet_for_safe_artifacts() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    let output = temp.path().join("packet");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join("demo-output.txt"), "claim table\n").unwrap();
    fs::write(
        artifacts.join("notes.md"),
        "# Notes\nSynthetic demo only.\n",
    )
    .unwrap();

    Command::cargo_bin("evidence-packet")
        .unwrap()
        .args([
            "create",
            artifacts.to_str().unwrap(),
            "--claim",
            "The CLI generated a claim review table from sample input.",
            "--scope",
            "local demo on synthetic markdown",
            "--limitations",
            "does not establish production readiness",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Evidence packet written"));

    let json_path = output.join("evidence_packet.json");
    let markdown_path = output.join("EVIDENCE_PACKET.md");
    assert!(json_path.exists());
    assert!(markdown_path.exists());

    let packet: Value = serde_json::from_str(&fs::read_to_string(json_path).unwrap()).unwrap();
    assert_eq!(
        packet["declared_claim"],
        "The CLI generated a claim review table from sample input."
    );
    assert_eq!(
        packet["supported_scope"],
        "local demo on synthetic markdown"
    );
    assert_eq!(packet["artifacts"].as_array().unwrap().len(), 2);
    assert_eq!(packet["artifacts"][0]["path"], "demo-output.txt");
    assert_eq!(packet["artifacts"][1]["path"], "notes.md");
    assert!(
        packet["artifacts"][0]["sha256"]
            .as_str()
            .unwrap()
            .chars()
            .all(|ch| ch.is_ascii_hexdigit())
    );
    assert_eq!(packet["artifacts"][0]["sha256"].as_str().unwrap().len(), 64);

    let markdown = fs::read_to_string(markdown_path).unwrap();
    assert!(markdown.contains("# Evidence Packet"));
    assert!(markdown.contains("This packet records supplied artifacts and declared boundaries."));
    assert!(markdown.contains("It does not prove that the declared claim is true."));
    assert!(markdown.contains("demo-output.txt"));
}

#[test]
fn create_refuses_risky_private_artifacts_by_default() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    let output = temp.path().join("packet");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join(".env"), "TOKEN=secret\n").unwrap();

    Command::cargo_bin("evidence-packet")
        .unwrap()
        .args([
            "create",
            artifacts.to_str().unwrap(),
            "--claim",
            "The demo has supporting artifacts.",
            "--scope",
            "local synthetic demo",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("risky artifact"));

    assert!(!output.join("evidence_packet.json").exists());
}

#[test]
fn allow_risky_records_warning_flags_instead_of_silently_accepting() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    let output = temp.path().join("packet");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join(".env"), "TOKEN=synthetic\n").unwrap();

    Command::cargo_bin("evidence-packet")
        .unwrap()
        .args([
            "create",
            artifacts.to_str().unwrap(),
            "--claim",
            "The demo has supporting artifacts.",
            "--scope",
            "local synthetic demo",
            "--output",
            output.to_str().unwrap(),
            "--allow-risky",
            "--risk-reviewed",
            "synthetic fixture intentionally included for risk-flag coverage",
        ])
        .assert()
        .success();

    let packet: Value =
        serde_json::from_str(&fs::read_to_string(output.join("evidence_packet.json")).unwrap())
            .unwrap();
    assert_eq!(packet["artifacts"][0]["path"], ".env");
    assert_eq!(
        packet["risk_review_note"],
        "synthetic fixture intentionally included for risk-flag coverage"
    );
    assert_eq!(
        packet["artifacts"][0]["risk_flags"][0],
        "possible_private_or_secret_file"
    );
}

#[test]
fn allow_risky_requires_review_note() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    let output = temp.path().join("packet");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join(".env.local"), "TOKEN=synthetic\n").unwrap();

    Command::cargo_bin("evidence-packet")
        .unwrap()
        .args([
            "create",
            artifacts.to_str().unwrap(),
            "--claim",
            "The demo has supporting artifacts.",
            "--scope",
            "local synthetic demo",
            "--output",
            output.to_str().unwrap(),
            "--allow-risky",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--risk-reviewed is required"));
}

#[test]
fn markdown_escapes_pipe_characters_in_artifact_paths() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    let output = temp.path().join("packet");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join("a|b.txt"), "sample\n").unwrap();

    Command::cargo_bin("evidence-packet")
        .unwrap()
        .args([
            "create",
            artifacts.to_str().unwrap(),
            "--claim",
            "The demo has supporting artifacts.",
            "--scope",
            "local synthetic demo",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let markdown = fs::read_to_string(output.join("EVIDENCE_PACKET.md")).unwrap();
    assert!(markdown.contains("`a\\|b.txt`"));
    assert!(!markdown.contains("`a|b.txt`"));
}
