use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnersRouteDataResponse {
    pub id: String,
    pub stats: Option<serde_json::Value>,
    pub user_stats: Option<UserStats>,
    pub user_rank: Option<UserRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub finished_at: String,
    pub finish_time: String,
    pub extra_stats: HashMap<String, String>,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRank {
    pub rank: i32,
    pub score: String,
    pub total: i64,
}
