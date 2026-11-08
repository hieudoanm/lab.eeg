use neuro_meg::formats::{detect_format, validate_file_set, FileDeclaration};

fn decl(name: &str, role: &str) -> FileDeclaration {
    FileDeclaration {
        name: name.to_string(),
        role: role.to_string(),
        size_bytes: 100,
    }
}

#[test]
fn file_set_validation_matches_reference_rules() {
    let findings = validate_file_set(
        "bin",
        &[decl("data.bin", "recording"), decl("header.json", "header")],
    );
    assert!(findings.is_empty());

    let findings = validate_file_set("fif", &[decl("rec.fif", "recording")]);
    assert!(findings.is_empty());

    let findings = validate_file_set("ctf", &[decl("data.bin", "recording")]);
    assert!(findings
        .iter()
        .any(|f| f.contains("missing required file role 'header'")));
    assert!(findings
        .iter()
        .any(|f| f.contains("missing required file role 'companion'")));

    let findings = validate_file_set("nope", &[decl("x", "recording")]);
    assert!(findings[0].contains("unsupported recording format"));

    let findings = validate_file_set(
        "kit",
        &[decl("a.sqd", "recording"), decl("b.sqd", "recording")],
    );
    assert!(findings.is_empty());
    let findings = validate_file_set(
        "kit",
        &[decl("a.sqd", "recording"), decl("a.sqd", "events")],
    );
    assert!(findings.iter().any(|f| f.contains("duplicate file name")));
}

#[test]
fn format_detection_prefers_specific_containers() {
    assert_eq!(detect_format(&["rec.fif".to_string()]), Some("fif"));
    assert_eq!(
        detect_format(&["header.json".to_string(), "data.bin".to_string()]),
        Some("bin")
    );
    assert_eq!(detect_format(&["random.txt".to_string()]), None);
}
