use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub persona_id: String,
    pub name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub tag: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagData {
    pub frame: Tag,
    pub bg: Tag,
    pub detail: Tag,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPlayerTagResponse {
    pub persona_id: String,
    pub tag_data: TagData,
}
