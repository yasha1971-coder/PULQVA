use pulqva_intent_http::build_interpretation_request;
use serde_json::{json, Value};

// Field-selection contract from the consumer we actually ship:
// llama.cpp b29c606e28a01b1bc8c1351026a0fa6e616bf6c4,
// tools/server/server-common.cpp, oaicompat_chat_params_parse (lines 1185-1204).
// For type=json_schema it reads json_schema.schema, NOT response_format.schema.
// This is a model-free protocol regression, not execution of upstream C++.
fn schema_selected_by_pinned_json_schema_branch(request: &Value) -> Value {
    assert_eq!(request["response_format"]["type"], "json_schema");
    let schema = request.pointer("/response_format/json_schema/schema")
        .cloned().unwrap_or_else(|| json!({}));
    if schema.as_object().is_some_and(|s| s.is_empty()) {
        json!({"type": "object"})
    } else {
        schema
    }
}

#[test]
fn request_embeds_versioned_schema_without_drift() {
    let schema_text = include_str!("../../../sidecars/llama.cpp/intent.schema.json");
    let schema: Value = serde_json::from_str(schema_text).unwrap();
    let text = build_interpretation_request("найди видео обратного отсчёта", schema_text).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["messages"][0]["role"], "user");
    assert_eq!(v["messages"][0]["content"], "найди видео обратного отсчёта");
    assert_eq!(v["temperature"], 0);
    assert_eq!(v["max_tokens"], 96);
    assert_eq!(v["chat_template_kwargs"]["enable_thinking"], false);
    assert_eq!(v["response_format"]["type"], "json_schema");
    assert_eq!(v["response_format"]["json_schema"]["name"], "pulqva_intent");
    assert_eq!(v["response_format"]["json_schema"]["strict"], true);
    assert!(v["response_format"].get("schema").is_none());
    // Full equality: required fields, enums and additionalProperties must all arrive.
    assert_eq!(schema_selected_by_pinned_json_schema_branch(&v), schema);
}

#[test]
fn legacy_top_level_schema_degrades_to_any_object() {
    let schema: Value = serde_json::from_str(
        include_str!("../../../sidecars/llama.cpp/intent.schema.json")
    ).unwrap();
    let legacy = json!({"response_format": {"type": "json_schema", "schema": schema.clone()}});
    let selected = schema_selected_by_pinned_json_schema_branch(&legacy);
    assert_eq!(selected, json!({"type": "object"}));
    assert_ne!(selected, schema);
}
