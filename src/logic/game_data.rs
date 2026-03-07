use crate::{
    entities::{
        ugc::{self, UgcType},
        user_ugc_flags, users,
    },
    logic::GatewayError,
    models::game_data::{LEVEL_ID_HASH, Transform, UgcId, UgcMeta},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

const UGC_BASE_URL: &str = "https://mec-gw.ops.dice.se/ugc/prod_default/prod_default/pc";

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
    reported: bool,
    blocked: bool,
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
                let url = format!(
                    "{}/{}/{}/{}",
                    UGC_BASE_URL, type_id, self.author_id, self.id
                );
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

pub struct BatchUgcLoader {
    authors: HashMap<i32, String>,
    flags: HashMap<Uuid, UgcFlags>,
}

impl BatchUgcLoader {
    pub async fn load<C>(
        db: &C,
        viewer_id: i32,
        ugc_entries: &[&ugc::Model],
    ) -> Result<Self, GatewayError>
    where
        C: ConnectionTrait,
    {
        if ugc_entries.is_empty() {
            return Ok(Self {
                authors: HashMap::new(),
                flags: HashMap::new(),
            });
        }

        let author_ids: Vec<i32> = ugc_entries.iter().map(|e| e.author_id).collect();
        let ugc_ids: Vec<Uuid> = ugc_entries.iter().map(|e| e.id).collect();

        let (authors_res, flags_res) = tokio::join!(
            users::Entity::find()
                .filter(users::Column::PersonaId.is_in(author_ids))
                .all(db),
            user_ugc_flags::Entity::find()
                .filter(user_ugc_flags::Column::UserId.eq(viewer_id))
                .filter(user_ugc_flags::Column::UgcId.is_in(ugc_ids))
                .all(db)
        );

        let authors_map = authors_res
            .map_err(GatewayError::from)?
            .into_iter()
            .map(|u| (u.persona_id, u.name))
            .collect();

        let flags_map = flags_res
            .map_err(GatewayError::from)?
            .into_iter()
            .map(|f| (f.ugc_id, UgcFlags::from(Some(f))))
            .collect();

        Ok(Self {
            authors: authors_map,
            flags: flags_map,
        })
    }

    pub fn get_author(&self, id: i32) -> &str {
        self.authors.get(&id).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_flag(&self, ugc_id: &Uuid) -> UgcFlags {
        self.flags.get(ugc_id).cloned().unwrap_or_default()
    }
}
