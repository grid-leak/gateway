use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagData {
    pub bg: TagItem,
    pub detail: TagItem,
    pub frame: TagItem,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagItem {
    pub tag: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerTagResponse {
    pub persona_id: String,
    pub tag_data: TagData,
}

// NOTE: we don't use the user-provided timestamp_value
// but it's good to have it here in case that changes in the future
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct GhostDataInput {
    pub customization: CustomizationInput,
    pub timestamp: TimestampInput,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationInput {
    pub variation: i32,
}

// NOTE: we don't use the user-provided timestamp_value
// but it's good to have it here in case that changes in the future
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct TimestampInput {
    pub timestamp_value: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGhost {
    pub persona_id: String,
    pub ghost_data: GhostDataOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GhostDataOutput {
    pub customization: CustomizationOutput,
    pub timestamp: TimestampOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationOutput {
    pub variation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampOutput {
    pub timestamp_value: String,
}
