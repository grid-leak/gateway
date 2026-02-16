use serde::{Deserialize, Serialize};

use crate::models::game_data::Transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReachThisMeta {
    pub level_id: u32,
    pub map_position: Transform,
    pub name: String,
    pub published: bool,
    pub transform: Transform,
}
