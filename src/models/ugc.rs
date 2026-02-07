// {
//   "data": "AAAAAwAAAAA=",
//   "meta": {
//     "levelId": 2354048661,
//     "mapPosition": {
//       "x": -307.223,
//       "y": 72.02,
//       "z": 208.48
//     },
//     "name": "ploxxxxxxy #1",
//     "published": true,
//     "transform": {
//       "qw": -0.0998407,
//       "qx": 0.0,
//       "qy": 0.995003,
//       "qz": 0.0,
//       "x": -309.633,
//       "y": 72.02,
//       "z": 196.19
//     }
//   }
// }

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
