use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct DesignInfo {
    pub name: String,
    pub view_index: ViewIndex,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct ViewIndex {
    pub updates_pending: UpdatesPending,
    pub waiting_commit: bool,
    pub waiting_clients: i64,
    pub updater_running: bool,
    pub update_seq: i64,
    pub sizes: Sizes,
    pub signature: String,
    pub purge_seq: i64,
    pub language: String,
    pub compact_running: bool,
    pub collator_versions: Vec<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct UpdatesPending {
    pub minimum: i64,
    pub preferred: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Sizes {
    pub file: i64,
    pub external: i64,
    pub active: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_design_info_response() {
        const SAMPLE_RESPONSE_1: &str = r#"{"name":"_test_design","view_index":{"updates_pending":{"minimum":0,"preferred":0,"total":0},"waiting_commit":false,"waiting_clients":0,"updater_running":false,"update_seq":31932,"sizes":{"file":1007952,"external":366478,"active":204834},"signature":"8176d6e264ec9d64e86016b50152e44c","purge_seq":0,"language":"query","compact_running":false,"collator_versions":["153.80"]}}"#;
        let design_info: DesignInfo = serde_json::from_str(SAMPLE_RESPONSE_1).expect("should parse");
        assert_eq!(design_info.name, "_test_design");
        assert_eq!(design_info.view_index.updates_pending.minimum, 0);
    }
}
