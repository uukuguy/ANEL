//! ANEL command spec tests — aligned with Go spec_test.go and Python test_spec.py
//!
//! Covers: All 12 command specs, JSON validity, schema structure,
//!         required fields, error codes, individual spec properties.

use qmd_rust::anel::*;

const ALL_COMMANDS: &[&str] = &[
    "search", "vsearch", "query", "get", "multi_get", "collection",
    "context", "embed", "update", "status", "cleanup", "agent", "mcp",
];

// ============================================================
// for_command — all commands
// ============================================================

#[test]
fn for_command_returns_spec_for_all_commands() {
    for cmd in ALL_COMMANDS {
        let spec = AnelSpec::for_command(cmd);
        assert!(spec.is_some(), "for_command({}) returned None", cmd);
        let spec = spec.unwrap();
        assert_eq!(spec.command, *cmd);
        assert_eq!(spec.version, ANEL_VERSION);
    }
}

#[test]
fn for_command_unknown_returns_none() {
    assert!(AnelSpec::for_command("nonexistent").is_none());
}

// ============================================================
// All specs — valid JSON
// ============================================================

#[test]
fn all_specs_to_json_valid() {
    for cmd in ALL_COMMANDS {
        let spec = AnelSpec::for_command(cmd).unwrap();
        let json_str = spec.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("Spec for {} produced invalid JSON: {}\n{}", cmd, e, json_str));

        for field in &["version", "command", "input_schema", "output_schema", "error_codes"] {
            assert!(parsed.get(field).is_some(), "Spec for {} missing field {}", cmd, field);
        }
    }
}

// ============================================================
// All specs — input schema is object with properties
// ============================================================

#[test]
fn all_specs_input_schema_is_object() {
    for cmd in ALL_COMMANDS {
        let spec = AnelSpec::for_command(cmd).unwrap();
        assert_eq!(
            spec.input_schema["type"], "object",
            "{} input_schema type should be object", cmd
        );
        assert!(
            spec.input_schema.get("properties").is_some(),
            "{} input_schema missing 'properties'", cmd
        );
    }
}

// ============================================================
// All specs — output schema is object
// ============================================================

#[test]
fn all_specs_output_schema_is_object() {
    for cmd in ALL_COMMANDS {
        let spec = AnelSpec::for_command(cmd).unwrap();
        assert_eq!(
            spec.output_schema["type"], "object",
            "{} output_schema type should be object", cmd
        );
    }
}

// ============================================================
// All specs — have error codes
// ============================================================

#[test]
fn all_specs_have_error_codes() {
    for cmd in ALL_COMMANDS {
        let spec = AnelSpec::for_command(cmd).unwrap();
        assert!(!spec.error_codes.is_empty(), "Spec for {} has no error codes", cmd);
    }
}

// ============================================================
// search spec
// ============================================================

#[test]
fn search_spec_requires_query() {
    let spec = AnelSpec::search();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "query"), "search should require 'query'");
}

#[test]
fn search_spec_error_codes() {
    let spec = AnelSpec::search();
    assert!(spec.error_codes.contains(&AnelErrorCode::SearchFailed));
    assert!(spec.error_codes.contains(&AnelErrorCode::IndexNotReady));
    assert!(spec.error_codes.contains(&AnelErrorCode::QueryParseError));
}

#[test]
fn search_spec_has_limit_property() {
    let spec = AnelSpec::search();
    assert!(spec.input_schema["properties"].get("limit").is_some());
}

#[test]
fn search_spec_has_min_score_property() {
    let spec = AnelSpec::search();
    assert!(spec.input_schema["properties"].get("min_score").is_some());
}

// ============================================================
// vsearch spec
// ============================================================

#[test]
fn vsearch_spec_requires_query() {
    let spec = AnelSpec::vsearch();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "query"));
}

#[test]
fn vsearch_spec_has_embedding_error() {
    let spec = AnelSpec::vsearch();
    assert!(spec.error_codes.contains(&AnelErrorCode::EmbeddingFailed));
}

#[test]
fn vsearch_spec_has_model_not_found() {
    let spec = AnelSpec::vsearch();
    assert!(spec.error_codes.contains(&AnelErrorCode::ModelNotFound));
}

// ============================================================
// query spec
// ============================================================

#[test]
fn query_spec_requires_query() {
    let spec = AnelSpec::query();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "query"));
}

#[test]
fn query_spec_output_has_reranked() {
    let spec = AnelSpec::query();
    let items = &spec.output_schema["properties"]["results"]["items"];
    assert!(items["properties"].get("reranked").is_some());
}

#[test]
fn query_spec_has_all_search_errors() {
    let spec = AnelSpec::query();
    assert!(spec.error_codes.contains(&AnelErrorCode::SearchFailed));
    assert!(spec.error_codes.contains(&AnelErrorCode::EmbeddingFailed));
    assert!(spec.error_codes.contains(&AnelErrorCode::QueryParseError));
}

// ============================================================
// get spec
// ============================================================

#[test]
fn get_spec_requires_file() {
    let spec = AnelSpec::get();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "file"));
}

#[test]
fn get_spec_output_has_lines() {
    let spec = AnelSpec::get();
    assert!(spec.output_schema["properties"].get("lines").is_some());
}

#[test]
fn get_spec_error_codes() {
    let spec = AnelSpec::get();
    assert!(spec.error_codes.contains(&AnelErrorCode::NotFound));
    assert!(spec.error_codes.contains(&AnelErrorCode::InvalidInput));
}

// ============================================================
// collection spec
// ============================================================

#[test]
fn collection_spec_has_actions() {
    let spec = AnelSpec::collection();
    let action = &spec.input_schema["properties"]["action"];
    let enums: Vec<&str> = action["enum"].as_array().unwrap()
        .iter().map(|v| v.as_str().unwrap()).collect();
    assert!(enums.contains(&"add"));
    assert!(enums.contains(&"list"));
    assert!(enums.contains(&"remove"));
    assert!(enums.contains(&"rename"));
}

#[test]
fn collection_spec_error_codes() {
    let spec = AnelSpec::collection();
    assert!(spec.error_codes.contains(&AnelErrorCode::CollectionNotFound));
    assert!(spec.error_codes.contains(&AnelErrorCode::CollectionExists));
    assert!(spec.error_codes.contains(&AnelErrorCode::InvalidInput));
}

// ============================================================
// context spec
// ============================================================

#[test]
fn context_spec_requires_action() {
    let spec = AnelSpec::context();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "action"));
}

#[test]
fn context_spec_has_actions() {
    let spec = AnelSpec::context();
    let action = &spec.input_schema["properties"]["action"];
    let enums: Vec<&str> = action["enum"].as_array().unwrap()
        .iter().map(|v| v.as_str().unwrap()).collect();
    assert!(enums.contains(&"add"));
    assert!(enums.contains(&"list"));
    assert!(enums.contains(&"rm"));
}

// ============================================================
// embed spec
// ============================================================

#[test]
fn embed_spec_has_force() {
    let spec = AnelSpec::embed();
    assert!(spec.input_schema["properties"].get("force").is_some());
}

#[test]
fn embed_spec_error_codes() {
    let spec = AnelSpec::embed();
    assert!(spec.error_codes.contains(&AnelErrorCode::EmbeddingFailed));
    assert!(spec.error_codes.contains(&AnelErrorCode::ModelNotFound));
    assert!(spec.error_codes.contains(&AnelErrorCode::ModelLoadFailed));
    assert!(spec.error_codes.contains(&AnelErrorCode::CollectionNotFound));
}

// ============================================================
// update spec
// ============================================================

#[test]
fn update_spec_has_pull() {
    let spec = AnelSpec::update();
    assert!(spec.input_schema["properties"].get("pull").is_some());
}

#[test]
fn update_spec_error_codes() {
    let spec = AnelSpec::update();
    assert!(spec.error_codes.contains(&AnelErrorCode::IndexNotReady));
    assert!(spec.error_codes.contains(&AnelErrorCode::CollectionNotFound));
    assert!(spec.error_codes.contains(&AnelErrorCode::StorageError));
}

// ============================================================
// status spec
// ============================================================

#[test]
fn status_spec_has_verbose() {
    let spec = AnelSpec::status();
    assert!(spec.input_schema["properties"].get("verbose").is_some());
}

// ============================================================
// cleanup spec
// ============================================================

#[test]
fn cleanup_spec_has_dry_run() {
    let spec = AnelSpec::cleanup();
    assert!(spec.input_schema["properties"].get("dry_run").is_some());
}

#[test]
fn cleanup_spec_has_older_than() {
    let spec = AnelSpec::cleanup();
    assert!(spec.input_schema["properties"].get("older_than").is_some());
}

// ============================================================
// agent spec
// ============================================================

#[test]
fn agent_spec_has_interactive() {
    let spec = AnelSpec::agent();
    assert!(spec.input_schema["properties"].get("interactive").is_some());
}

#[test]
fn agent_spec_has_mcp_option() {
    let spec = AnelSpec::agent();
    assert!(spec.input_schema["properties"].get("mcp").is_some());
}

// ============================================================
// mcp spec
// ============================================================

#[test]
fn mcp_spec_has_transport() {
    let spec = AnelSpec::mcp();
    assert!(spec.input_schema["properties"].get("transport").is_some());
}

#[test]
fn mcp_spec_has_port() {
    let spec = AnelSpec::mcp();
    assert!(spec.input_schema["properties"].get("port").is_some());
}

#[test]
fn mcp_spec_error_codes() {
    let spec = AnelSpec::mcp();
    assert!(spec.error_codes.contains(&AnelErrorCode::ConfigError));
    assert!(spec.error_codes.contains(&AnelErrorCode::BackendUnavailable));
}

// ============================================================
// multi_get spec
// ============================================================

#[test]
fn multi_get_spec_requires_pattern() {
    let spec = AnelSpec::multi_get();
    let required = spec.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "pattern"));
}

#[test]
fn multi_get_spec_output_has_files() {
    let spec = AnelSpec::multi_get();
    assert!(spec.output_schema["properties"].get("files").is_some());
}
