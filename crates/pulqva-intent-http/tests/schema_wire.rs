//! Wire conformance only. These tests do not execute llama.cpp or a model.
use pulqva_intent_http::{build_interpretation_request, IntentHttpError};
use serde::Deserialize;
use serde_json::{value::RawValue, Value};

const SCHEMA: &str = include_str!("../../../sidecars/llama.cpp/intent.schema.json");

// Decode the actual outgoing field without normalizing its object keys again.
#[derive(Deserialize)]
struct WireRequest { response_format: WireFormat }
#[derive(Deserialize)]
struct WireFormat { json_schema: WireSchema }
#[derive(Deserialize)]
struct WireSchema { schema: Box<RawValue> }
#[derive(Deserialize)]
struct Branches { #[serde(rename = "oneOf")] alternatives: Vec<Branch> }
#[derive(Deserialize)]
struct Branch { properties: Box<RawValue> }

fn schema_on_wire(request: &str) -> Box<RawValue> {
    let parsed: WireRequest = serde_json::from_str(request).unwrap();
    parsed.response_format.json_schema.schema
}

#[test]
fn normative_schema_bytes_and_kind_first_branches_reach_the_wire() {
    let request = build_interpretation_request("find ocean recordings", SCHEMA).unwrap();
    let raw = schema_on_wire(&request);
    // JSON-value bytes are exact; whitespace outside the value is not part of it.
    assert_eq!(raw.get(), SCHEMA.trim());
    let sent: Value = serde_json::from_str(&request).unwrap();
    let expected: Value = serde_json::from_str(SCHEMA).unwrap();
    assert_eq!(sent.pointer("/response_format/json_schema/schema"), Some(&expected));
    assert!(sent["response_format"]["json_schema"]["schema"].is_object());

    let branches: Branches = serde_json::from_str(raw.get()).unwrap();
    assert_eq!(branches.alternatives.len(), 2);
    let intent = branches.alternatives[0].properties.get();
    let kind = intent.find("\"kind\"").unwrap();
    let query = intent.find("\"query\"").unwrap();
    let mode = intent.find("\"choice_mode\"").unwrap();
    assert!(kind < query && query < mode);
    let reject = branches.alternatives[1].properties.get();
    assert!(reject.find("\"kind\"").unwrap() < reject.find("\"reason\"").unwrap());
}

#[test]
fn equal_schemas_keep_their_distinct_source_order_and_escaping() {
    // Deliberately equal as Values, unequal as bytes. No production schema edit.
    let a = r#"{ "type":"object", "properties":{"z":{"const":"\u0430"},"a":{"type":"string"}}, "required":["z","a"] }"#;
    let b = r#"{"required":["z","a"],"properties":{"a":{"type":"string"},"z":{"const":"а"}},"type":"object"}"#;
    assert_eq!(serde_json::from_str::<Value>(a).unwrap(), serde_json::from_str::<Value>(b).unwrap());
    for original in [a, b] {
        let padded = format!(" \r\n\t{original}\n");
        let request = build_interpretation_request("find birds", &padded).unwrap();
        assert_eq!(schema_on_wire(&request).get(), original);
    }
}

#[test]
fn malformed_or_multiple_values_cannot_escape_the_schema_field() {
    for invalid in [
        "", "{", "{}{}", "{} trailing", "{} ,\"messages\":[]",
        r#"{"type":"object","minimum":1e400}"#,
        r#"{"title":"\uD800"}"#,
    ] {
        assert!(matches!(build_interpretation_request("find birds", invalid), Err(IntentHttpError::InvalidPlan)));
    }
    // Retain Value's recursion-limit check, which RawValue alone need not impose.
    let nested = format!("{}null{}", "[".repeat(256), "]".repeat(256));
    assert!(matches!(build_interpretation_request("find birds", &nested), Err(IntentHttpError::InvalidPlan)));
}
