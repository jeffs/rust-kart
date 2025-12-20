//! Integration tests for analyzing the sample crate

use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_sample_crate_has_seven_edges() {
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    // Count edges
    let edge_count = graph.edges().count();
    assert_eq!(edge_count, 7, "Expected 7 edges, got {edge_count}");
}

#[test]
fn test_sample_crate_has_five_modules() {
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    // Count modules
    let module_count = graph.modules().count();
    assert_eq!(module_count, 5, "Expected 5 modules, got {module_count}");
}

#[test]
fn test_sample_crate_mermaid_output() {
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    let mermaid = graph.to_mermaid();

    // Verify structure
    assert!(mermaid.starts_with("flowchart TD\n"));

    // Verify all expected nodes are present
    assert!(mermaid.contains(r#"crate["crate"]"#));
    assert!(mermaid.contains(r#"alpha["alpha"]"#));
    assert!(mermaid.contains(r#"alpha_delta["alpha::delta"]"#));
    assert!(mermaid.contains(r#"beta["beta"]"#));
    assert!(mermaid.contains(r#"gamma["gamma"]"#));

    // Verify key edges
    assert!(mermaid.contains("crate --> alpha"));
    assert!(mermaid.contains("crate --> beta"));
    assert!(mermaid.contains("crate --> gamma"));
    assert!(mermaid.contains("alpha --> alpha_delta"));
    assert!(mermaid.contains("beta --> alpha"));
    assert!(mermaid.contains("beta --> gamma"));
    assert!(mermaid.contains("gamma --> crate"));
}

#[test]
fn test_cycle_detected() {
    // The sample crate has a cycle: gamma -> crate (via `use super::Root`)
    // Verify this edge exists alongside crate -> gamma
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    let mermaid = graph.to_mermaid();

    // Both directions of the cycle should be present
    assert!(
        mermaid.contains("crate --> gamma"),
        "Missing crate -> gamma edge"
    );
    assert!(
        mermaid.contains("gamma --> crate"),
        "Missing gamma -> crate edge (cycle)"
    );
}
