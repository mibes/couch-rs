use serde::{Deserialize, Serialize};

/// Membership state of a CouchDB cluster.
/// See [_membership](https://docs.couchdb.org/en/latest/api/server/common.html?#membership) for more details.
#[derive(Deserialize, Debug)]
pub struct Membership {
    pub cluster_nodes: Vec<String>,
    pub all_nodes: Vec<String>,
}

/// Cluster setup state of a CouchDB cluster.
/// See [_cluster_setup](https://docs.couchdb.org/en/latest/api/server/common.html?#cluster-setup) for more details.
#[derive(Deserialize, Debug)]
pub struct ClusterSetupGetResponse {
    pub state: ClusterSetup,
}

#[derive(Serialize, Debug)]
pub struct EnsureDbsExist {
    pub ensure_dbs_exist: Vec<String>,
}

impl EnsureDbsExist {
    pub fn with_dbs(ensure_dbs_exist: Vec<String>) -> Self {
        Self { ensure_dbs_exist }
    }
}

impl Default for EnsureDbsExist {
    fn default() -> Self {
        Self::with_dbs(vec!["_users".to_string(), "_replicator".to_string()])
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ClusterSetup {
    ClusterDisabled,
    SingleNodeDisabled,
    SingleNodeEnabled,
    ClusterEnabled,
    ClusterFinished,
}
