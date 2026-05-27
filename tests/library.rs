use evidence_packet::{CreatePacketOptions, create_packet};
use std::fs;
use tempfile::tempdir;

#[test]
fn scan_orders_artifacts_by_relative_path() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    fs::create_dir_all(artifacts.join("nested")).unwrap();
    fs::write(artifacts.join("zeta.txt"), "z\n").unwrap();
    fs::write(artifacts.join("alpha.txt"), "a\n").unwrap();
    fs::write(artifacts.join("nested").join("middle.txt"), "m\n").unwrap();

    let packet = create_packet(CreatePacketOptions {
        artifact_dir: artifacts,
        declared_claim: "A local demo produced sample output.".to_string(),
        supported_scope: "local demo".to_string(),
        limitations: vec!["synthetic data only".to_string()],
        allow_risky: false,
        risk_review_note: None,
    })
    .unwrap();

    let paths: Vec<_> = packet
        .artifacts
        .iter()
        .map(|artifact| artifact.path.as_str())
        .collect();
    assert_eq!(paths, vec!["alpha.txt", "nested/middle.txt", "zeta.txt"]);
}

#[test]
fn packet_has_boundary_note_that_does_not_overclaim_verification() {
    let temp = tempdir().unwrap();
    let artifacts = temp.path().join("artifacts");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join("artifact.txt"), "sample\n").unwrap();

    let packet = create_packet(CreatePacketOptions {
        artifact_dir: artifacts,
        declared_claim: "The local example ran.".to_string(),
        supported_scope: "one local sample run".to_string(),
        limitations: vec![],
        allow_risky: false,
        risk_review_note: None,
    })
    .unwrap();

    assert!(packet.boundary_note.contains("does not prove"));
    assert!(packet.boundary_note.contains("does not verify truth"));
}
