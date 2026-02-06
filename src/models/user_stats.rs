use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntryUserStats {
    RunnersRoute(RunnersRouteUserStats),
    ReachThis(ReachThisUserStats),
    HackableBillboard(HackableBillboardUserStats),
    TimeTrial(TimeTrialUserStats),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HackableBillboardUserStats {
    pub finished_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachThisUserStats {
    pub reached_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnersRouteUserStats {
    pub finished_at: String,
    pub finish_time: String,
    pub extra_stats: HashMap<String, String>,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeTrialUserStats {
    pub finish_time: String,
    pub split_times: Vec<String>,
    pub extra_stats: HashMap<String, String>,
}
