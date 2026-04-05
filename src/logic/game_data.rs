use crate::{
    entities::{
        ugc::{self, UgcType},
        user_ugc_flags,
    },
    logic::GatewayError,
    models::game_data::{LEVEL_ID_HASH, Transform, UgcId, UgcMeta},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::env;
use std::fmt;
use uuid::Uuid;

impl fmt::Display for UgcType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            UgcType::ReachThis => "ReachThis",
            UgcType::TimeTrial => "TimeTrial",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UgcFlags {
    pub reported: bool,
    pub blocked: bool,
}

impl From<Option<user_ugc_flags::Model>> for UgcFlags {
    fn from(model: Option<user_ugc_flags::Model>) -> Self {
        match model {
            Some(m) => Self {
                reported: m.reported,
                blocked: m.blocked,
            },
            None => Self::default(),
        }
    }
}

impl ugc::Model {
    pub fn into_meta(self, author_name: &str, flags: &UgcFlags) -> UgcMeta {
        let type_id = self.r#type.to_string();

        let transform = Transform {
            x: self.x,
            y: self.y,
            z: self.z,
            qx: Some(self.qx),
            qy: Some(self.qy),
            qz: Some(self.qz),
            qw: Some(self.qw),
        };

        let (map_position, teleport_transform, ugc_url) = match self.r#type {
            UgcType::ReachThis => {
                let pos = Transform {
                    x: self.x,
                    y: self.y,
                    z: self.z,
                    qx: None,
                    qy: None,
                    qz: None,
                    qw: None,
                };
                (Some(pos), None, None)
            }
            UgcType::TimeTrial => {
                let base_url = env::var("UGC_BASE_URL").expect("UGC_BASE_URL must be set");
                let url = format!("{}/{}", base_url, self.id);
                (None, Some(transform.clone()), Some(url))
            }
        };

        UgcMeta {
            ugc_id: UgcId {
                user_id: self.author_id,
                id: self.id.to_string(),
            },
            name: self.name,
            creator_name: author_name.to_string(),
            created_at: self.created_at.timestamp_millis().to_string(),
            updated_at: self.updated_at.timestamp_millis().to_string(),
            published: self.published,
            reported: flags.reported,
            blocked: flags.blocked,
            level_id: LEVEL_ID_HASH,
            transform,
            map_position,
            teleport_transform,
            ugc_url,
            type_id: type_id.to_string(),
        }
    }
}

pub async fn load_ugc_flags<C>(
    db: &C,
    viewer_id: i32,
    ugc_ids: &[Uuid],
) -> Result<HashMap<Uuid, UgcFlags>, GatewayError>
where
    C: ConnectionTrait,
{
    if ugc_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let flags = user_ugc_flags::Entity::find()
        .filter(user_ugc_flags::Column::UserId.eq(viewer_id))
        .filter(user_ugc_flags::Column::UgcId.is_in(ugc_ids.to_vec()))
        .all(db)
        .await?;

    Ok(flags
        .into_iter()
        .map(|f| (f.ugc_id, UgcFlags::from(Some(f))))
        .collect())
}
