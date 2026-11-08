use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDeclaration {
    pub name: String,
    pub role: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct FileSetRequirement {
    pub format: &'static str,
    pub required_roles: &'static [&'static str],
    pub optional_roles: &'static [&'static str],
}

pub const FORMAT_REQUIREMENTS: &[FileSetRequirement] = &[
    FileSetRequirement {
        format: "fif",
        required_roles: &["recording"],
        optional_roles: &["events", "channels", "coordsystem", "metadata"],
    },
    FileSetRequirement {
        format: "ctf",
        required_roles: &["header", "recording", "companion"],
        optional_roles: &[
            "events",
            "coordsystem",
            "calibration",
            "crosstalk",
            "metadata",
        ],
    },
    FileSetRequirement {
        format: "kit",
        required_roles: &["recording"],
        optional_roles: &[
            "header",
            "coordsystem",
            "calibration",
            "crosstalk",
            "events",
            "metadata",
        ],
    },
    FileSetRequirement {
        format: "bin",
        required_roles: &["recording", "header"],
        optional_roles: &["events", "coordsystem", "calibration", "metadata"],
    },
    FileSetRequirement {
        format: "bids",
        required_roles: &["metadata", "recording"],
        optional_roles: &["events", "channels", "coordsystem", "header", "companion"],
    },
];

pub fn supported_formats() -> Vec<&'static str> {
    FORMAT_REQUIREMENTS.iter().map(|r| r.format).collect()
}

pub fn default_acquisition_system(recording_format: &str) -> &'static str {
    if recording_format == "bin" {
        "opm"
    } else {
        "squid"
    }
}

/// Returns validation findings; an empty list means the file set is complete.
pub fn validate_file_set(recording_format: &str, files: &[FileDeclaration]) -> Vec<String> {
    let Some(requirement) = FORMAT_REQUIREMENTS
        .iter()
        .find(|r| r.format == recording_format)
    else {
        return vec![format!("unsupported recording format: {recording_format}")];
    };
    let mut findings = Vec::new();
    let mut seen_roles = BTreeSet::new();
    let mut names = BTreeSet::new();
    for declaration in files {
        if !names.insert(declaration.name.clone()) {
            findings.push(format!("duplicate file name: {}", declaration.name));
        }
        seen_roles.insert(declaration.role.clone());
    }
    for role in requirement.required_roles {
        if !seen_roles.contains(*role) {
            findings.push(format!(
                "missing required file role '{role}' for format '{recording_format}'"
            ));
        }
    }
    for role in &seen_roles {
        if !requirement.required_roles.contains(&role.as_str())
            && !requirement.optional_roles.contains(&role.as_str())
        {
            findings.push(format!(
                "unexpected file role '{role}' for format '{recording_format}'"
            ));
        }
    }
    if files.iter().map(|f| f.size_bytes).sum::<u64>() == 0 {
        findings.push("declared file set is empty".into());
    }
    findings
}

/// Infer a recording format from the file names present in a directory.
pub fn detect_format(file_names: &[String]) -> Option<&'static str> {
    let has_suffix = |suffix: &str| file_names.iter().any(|n| n.ends_with(suffix));
    let has_name = |name: &str| file_names.iter().any(|n| n == name);
    if has_suffix(".fif") {
        Some("fif")
    } else if has_name("header.bin") && has_name("data.bin") {
        Some("ctf")
    } else if has_suffix(".sqd") || has_suffix(".con") || has_suffix(".sqc") {
        Some("kit")
    } else if (has_name("header.json") || has_name("channels.json"))
        && file_names.iter().any(|n| n.ends_with(".bin"))
    {
        Some("bin")
    } else if has_name("dataset_description.json") {
        Some("bids")
    } else {
        None
    }
}
