use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EntryUserStats {
    HackableBillboard(HackableBillboardUserStats),
    RunnersRoute(RunnersRouteUserStats),
    ReachThis(ReachThisUserStats),
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
    pub finish_time: i32,
    pub extra_stats: HashMap<String, String>,
    pub run_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeTrialUserStats {
    pub finish_time: i32,
    pub split_times: Vec<String>,
    pub extra_stats: HashMap<String, String>,
}
