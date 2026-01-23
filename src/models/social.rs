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
