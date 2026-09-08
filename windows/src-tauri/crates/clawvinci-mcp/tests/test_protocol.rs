// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_mcp::protocol::{
    all_tool_definitions, error_codes, JsonRpcRequest, JsonRpcResponse,
};
use serde_json::json;
use std::collections::HashSet;

#[test]
fn test_json_rpc_serialization() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };
    let json_str = serde_json::to_string(&req).expect("Failed to serialize request");
    let deserialized: JsonRpcRequest =
        serde_json::from_str(&json_str).expect("Failed to deserialize request");
    assert_eq!(deserialized.method, "tools/list");
    assert_eq!(deserialized.id, Some(json!(1)));

    let resp = JsonRpcResponse::success(Some(json!(1)), json!({ "status": "ok" }));
    let resp_str = serde_json::to_string(&resp).expect("Failed to serialize response");
    assert!(resp_str.contains("\"result\":{\"status\":\"ok\"}"));

    let err_resp = JsonRpcResponse::error(Some(json!(2)), error_codes::METHOD_NOT_FOUND, "Unknown");
    let err_str = serde_json::to_string(&err_resp).expect("Failed to serialize error response");
    assert!(err_str.contains("-32601"));
}

#[test]
fn test_all_tool_definitions_catalog() {
    let tools = all_tool_definitions();
    assert_eq!(
        tools.len(),
        52,
        "Expected exactly 52 tools in the MCP catalog"
    );

    let mut names = HashSet::new();
    for tool in &tools {
        assert!(
            names.insert(tool.name.clone()),
            "Duplicate tool name found: {}",
            tool.name
        );
        assert!(
            !tool.description.trim().is_empty(),
            "Tool {} description is empty",
            tool.name
        );
        assert_eq!(
            tool.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "Tool {} schema must be of type 'object'",
            tool.name
        );
    }

    // Verify key domain tools exist
    assert!(names.contains("get_timeline"));
    assert!(names.contains("add_clips"));
    assert!(names.contains("split_clips"));
    assert!(names.contains("move_clips"));
    assert!(names.contains("undo"));
    assert!(names.contains("manage_markers"));
    assert!(names.contains("manage_project"));
    assert!(names.contains("export_project"));
    assert!(names.contains("apply_color"));
    assert!(names.contains("apply_effect"));
}
