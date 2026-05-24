use crate::{
    logic::GatewayError,
    models::game_data::{LEVEL_ID_HASH, Transform, UgcId, UgcMeta},
};
use entities::{
    ugc::{self, UgcType},
    user_ugc_flags,
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use uuid::Uuid;

static UGC_BASE_URL: OnceLock<String> = OnceLock::new();

pub fn ugc_type_to_string(t: &UgcType) -> String {
    match t {
        UgcType::ReachThis => "ReachThis".to_string(),
        UgcType::TimeTrial => "TimeTrial".to_string(),
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

pub fn ugc_to_meta(model: ugc::Model, author_name: &str, flags: &UgcFlags) -> UgcMeta {
    let type_id = ugc_type_to_string(&model.r#type);

    let transform = Transform {
        x: model.x,
        y: model.y,
        z: model.z,
        qx: Some(model.qx),
        qy: Some(model.qy),
        qz: Some(model.qz),
        qw: Some(model.qw),
    };

    let (map_position, teleport_transform, ugc_url) = match model.r#type {
        UgcType::ReachThis => {
            let pos = Transform {
                x: model.x,
                y: model.y,
                z: model.z,
                qx: None,
                qy: None,
                qz: None,
                qw: None,
            };
            (Some(pos), None, None)
        }
        UgcType::TimeTrial => {
            let base_url = UGC_BASE_URL.get_or_init(|| {
                env::var("UGC_BASE_URL").expect("UGC_BASE_URL must be set")
            });
            let url = format!("{}/{}", base_url, model.id);
            (None, Some(transform.clone()), Some(url))
        }
    };

    UgcMeta {
        ugc_id: UgcId {
            user_id: model.author_id,
            id: model.id.to_string(),
        },
        name: model.name,
        creator_name: author_name.to_string(),
        created_at: model.created_at.timestamp_millis().to_string(),
        updated_at: model.updated_at.timestamp_millis().to_string(),
        published: model.published,
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
