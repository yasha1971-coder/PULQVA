use pulqva_intent_http::build_interpretation_request;
#[test]
fn request_embeds_versioned_schema_without_drift() {
    let schema=include_str!("../../../sidecars/llama.cpp/intent.schema.json");
    let text=build_interpretation_request("найди видео обратного отсчёта",schema).unwrap();
    let v:serde_json::Value=serde_json::from_str(&text).unwrap();
    assert_eq!(v["messages"][0]["role"],"user");
    assert_eq!(v["messages"][0]["content"],"найди видео обратного отсчёта");
    assert_eq!(v["temperature"],0);
    assert_eq!(v["response_format"]["type"],"json_schema");
    assert_eq!(v["response_format"]["schema"]["oneOf"][0]["properties"]["kind"]["const"],"intent");
    assert_eq!(v["response_format"]["schema"]["oneOf"][1]["properties"]["reason"]["const"],"semantic_authority");
}
