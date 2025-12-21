//! Integration tests for analyzing the sample crate

use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_sample_crate_module_count() {
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    // Modules: crate, alpha, alpha::delta, alpha::tests, beta, gamma, tests
    let module_count = graph.modules().count();
    assert_eq!(module_count, 7, "Expected 7 modules, got {module_count}");
}

#[test]
fn test_sample_crate_edge_count() {
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    // 6 mod declarations + 5 use imports = 11 edges
    let edge_count = graph.edges().count();
    assert_eq!(edge_count, 11, "Expected 11 edges, got {edge_count}");
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
    assert!(mermaid.contains(r#"alpha_tests["alpha::tests"]"#));
    assert!(mermaid.contains(r#"beta["beta"]"#));
    assert!(mermaid.contains(r#"gamma["gamma"]"#));
    assert!(mermaid.contains(r#"tests["tests"]"#));

    // Verify mod declaration edges (solid arrows)
    assert!(mermaid.contains("crate --> alpha"));
    assert!(mermaid.contains("crate --> beta"));
    assert!(mermaid.contains("crate --> gamma"));
    assert!(mermaid.contains("crate --> tests"));
    assert!(mermaid.contains("alpha --> alpha_delta"));
    assert!(mermaid.contains("alpha --> alpha_tests"));

    // Verify use import edges (dashed arrows)
    assert!(mermaid.contains("beta -.-> alpha"));
    assert!(mermaid.contains("beta -.-> gamma"));
    assert!(mermaid.contains("gamma -.-> crate"));
    assert!(mermaid.contains("tests -.-> crate"));
    assert!(mermaid.contains("alpha_tests -.-> alpha"));
}

#[test]
fn test_cycle_detected() {
    // The sample crate has a cycle: gamma -> crate (via `use super::Root`)
    // Verify this edge exists alongside crate -> gamma
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    let mermaid = graph.to_mermaid();

    // mod declaration: crate contains gamma (solid arrow)
    assert!(
        mermaid.contains("crate --> gamma"),
        "Missing crate -> gamma edge"
    );
    // use import: gamma imports from crate (dashed arrow, cycle!)
    assert!(
        mermaid.contains("gamma -.-> crate"),
        "Missing gamma -> crate edge (cycle)"
    );
}

#[test]
fn test_edge_kind_distinction() {
    // Verify that mod declarations use solid arrows and use imports use dashed arrows
    let sample_path = fixtures_dir().join("sample");
    let graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    let mermaid = graph.to_mermaid();

    // Count solid arrows (mod declarations)
    let solid_count = mermaid.matches(" --> ").count();
    assert_eq!(
        solid_count, 6,
        "Expected 6 mod declaration edges (solid arrows)"
    );

    // Count dashed arrows (use imports)
    let dashed_count = mermaid.matches(" -.-> ").count();
    assert_eq!(
        dashed_count, 5,
        "Expected 5 use import edges (dashed arrows)"
    );
}

#[test]
fn test_exclude_tests_modules() {
    let sample_path = fixtures_dir().join("sample");
    let mut graph =
        dgmod::analyze_crate(&sample_path, "sample").expect("Failed to analyze sample crate");

    // Before excluding: should have tests modules
    let has_tests_before = graph.modules().any(|m| m.path.as_str() == "tests");
    let has_alpha_tests_before = graph.modules().any(|m| m.path.as_str() == "alpha::tests");
    assert!(has_tests_before, "should have a tests module");
    assert!(has_alpha_tests_before, "should have an alpha::tests module");

    // Before: 7 modules, 11 edges
    assert_eq!(graph.modules().count(), 7);
    assert_eq!(graph.edges().count(), 11);

    // Exclude tests modules
    graph.exclude_tests_modules();

    // After excluding: tests modules should be gone
    let has_tests_after = graph.modules().any(|m| m.path.as_str() == "tests");
    let has_alpha_tests_after = graph.modules().any(|m| m.path.as_str() == "alpha::tests");
    assert!(!has_tests_after, "tests module should be excluded");
    assert!(
        !has_alpha_tests_after,
        "alpha::tests module should be excluded"
    );

    // After: 5 modules (crate, alpha, alpha::delta, beta, gamma)
    assert_eq!(graph.modules().count(), 5);

    // After: 7 edges (removed 2 mod declarations + 2 use imports = 4 edges)
    assert_eq!(graph.edges().count(), 7);

    // Verify no tests-related content in output
    let mermaid = graph.to_mermaid();
    assert!(
        !mermaid.contains("tests"),
        "No tests-related content should appear in output"
    );

    // But other modules should still exist
    assert!(mermaid.contains(r#"alpha["alpha"]"#));
    assert!(mermaid.contains(r#"alpha_delta["alpha::delta"]"#));
}
