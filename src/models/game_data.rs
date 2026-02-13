use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{DisplayFromStr, PickFirst, serde_as};

use crate::models::user_stats::{
    HackableBillboardUserStats, ReachThisUserStats, RunnersRouteUserStats, TimeTrialUserStats,
};

pub const LEVEL_ID_HASH: i32 = djb_hash("SP_MainCity");

const fn djb_hash(s: &str) -> i32 {
    let mut hash: u32 = 5381;
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        hash = (bytes[i] as u32) ^ hash.wrapping_mul(33);
        i += 1;
    }

    hash as i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialGameDataResponse {
    pub player_info: PlayerInfo,
    pub user_stats: serde_json::Value,
    pub user_reach_this: Vec<UgcWrapper>,
    pub user_time_trials: Vec<UgcWrapper>,
    #[serde(rename = "promotedUGC")]
    pub promoted_ugc: Vec<PromotedUgcWrapper>,
    pub bookmarks: Bookmarks,
    pub inventory: Inventory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub name: String,
    pub location: Vec<Location>,
    pub division: Division,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub r#type: String,
    pub name: String,
    pub cc: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Division {
    pub name: String,
    pub rank: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UgcWrapper {
    pub meta: UgcMeta,
    pub stats: Option<()>,
    pub user_stats: Option<()>,
    pub user_rank: Option<()>,
}

// TODO: make universal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachThisWrapper {
    pub meta: Option<UgcMeta>,
    pub stats: Option<()>,
    pub user_stats: Option<ReachThisUserStats>,
    pub user_rank: Option<UserRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotedUgcWrapper {
    pub meta: UgcMeta,
    pub reason: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UgcMeta {
    pub ugc_id: UgcId,
    pub name: String,
    pub creator_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub published: bool,
    pub reported: bool,
    pub blocked: bool,
    pub level_id: i32,
    pub transform: Transform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_position: Option<Transform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teleport_transform: Option<Transform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ugc_url: Option<String>,
    pub type_id: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UgcId {
    // Deserializes from string or i32, serializes into string
    #[serde_as(as = "PickFirst<(DisplayFromStr, _)>")]
    pub user_id: i32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qx: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmarks {
    pub ugc_bookmarks: Vec<UgcBookmarkEntry>,
    pub challenge_bookmarks: Vec<ChallengeBookmarkEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UgcBookmarkEntry {
    pub ugc_type: String,
    pub bookmark_time: String,
    pub meta: UgcMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeBookmarkEntry {
    pub challenge_id: String,
    pub bookmark_time: String,
    pub challenge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub kits: Vec<Kit>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kit {
    pub id: String,
    pub kit_type: String,
    pub opened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HackableBillboardLeader {
    pub position: i32,
    pub score: String,
    pub persona_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnersRouteData {
    pub id: String,
    pub stats: Option<Value>,
    pub user_stats: Option<RunnersRouteUserStats>,
    pub user_rank: Option<UserRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRank {
    pub rank: i32,
    pub score: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Ugc(UgcEntry),
    Challenge(ChallengeEntry),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "typeId")]
pub enum UgcEntry {
    ReachThis {
        #[serde(rename = "ugcId")]
        ugc_id: UgcId,
        stats: ReachThisUserStats,
    },
    TimeTrial {
        #[serde(rename = "ugcId")]
        ugc_id: UgcId,
        stats: TimeTrialUserStats,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "challengeType")]
pub enum ChallengeEntry {
    HackableBillboard {
        #[serde(rename = "challengeId")]
        challenge_id: String,
        stats: HackableBillboardUserStats,
    },
    RunnersRoute {
        #[serde(rename = "challengeId")]
        challenge_id: String,
        stats: RunnersRouteUserStats,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUgcLimits {
    pub ugc_count: i32,
    pub max_ugc: i32,
    pub published_count: i32,
    pub max_published: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewReachThisLeaderboardResponse {
    pub leaderboard: LeaderboardWrapper,
    pub global_leader: Option<LeaderboardUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub leaderboard: LeaderboardWrapper,
    pub global_leader: Option<LeaderboardUser>,
    pub global_count: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardWrapper {
    pub area: Option<()>,
    pub total_count: i64,
    pub users: Vec<LeaderboardUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardUser {
    pub position: i32,
    pub global_rank: i32,
    pub score: String,
    pub percentile: Option<f64>,
    pub persona_id: String,
    pub name: String,
    pub division: Division,
}
