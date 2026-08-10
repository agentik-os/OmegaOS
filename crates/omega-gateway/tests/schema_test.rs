#[test]
fn schema_contains_all_wire_types() {
    let schema = omega_gateway::protocol::schema_json();
    let v: serde_json::Value = serde_json::from_str(&schema).unwrap();
    let defs = v["definitions"].as_object().or_else(|| v["$defs"].as_object()).unwrap();
    for ty in [
        "PairRequest",
        "PairResponse",
        "SessionsResponse",
        "StreamFrame",
        "WhoamiResponse",
        "ChatMeta",
        "ChatMessage",
        "ChatAgent",
        "ChatStreamServerMsg",
        "ChatStreamClientMsg",
        "Mission",
        "MissionTask",
        "GatewayEvent",
        "Account",
        "AccountKind",
        "AccountWithStatus",
        "AccountCreateRequest",
        "ApiKeyRequest",
        "AccountLoginServerMsg",
        "LawEntry",
        "RuleEntry",
        "RulesResponse",
        "AgentEntry",
        "AgentsResponse",
    ] {
        assert!(defs.contains_key(ty), "missing {ty} in schema");
    }
}
