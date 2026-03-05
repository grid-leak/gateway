use crate::{
    context::GatewayContext,
    entities::{challenge_entries, ugc, ugc_entries},
    entities::{users, users::Entity as Users},
    logic::GatewayError,
    models::customization::{
        CustomizationOutput, GhostDataInput, GhostDataOutput, PlayerGhost, TagData, TimestampOutput,
    },
    models::game_data::{
        ChallengeEntry, Division, Entry, PlayerInfo, PlayerUgcLimits, UgcEntry, UgcId,
    },
};
use chrono::Utc;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

pub async fn set_player_ghost(
    ctx: &GatewayContext,
    persona_id: i32,
    data: GhostDataInput,
) -> Result<(), GatewayError> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or_else(|| GatewayError::internal("user not found"))?;

    let mut user: users::ActiveModel = user.into();

    // ignore the provided timestamp and set the current time
    let timestamp = Utc::now().timestamp();

    user.ghost_data = Set(serde_json::json!({
        "variation": data.customization.variation,
        "timestamp": timestamp,
    }));

    user.update(ctx.db()).await?;

    Ok(())
}

pub async fn set_player_tag(
    ctx: &GatewayContext,
    persona_id: i32,
    tag_data: TagData,
) -> Result<(), GatewayError> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or_else(|| GatewayError::internal("user not found"))?;

    let mut user: users::ActiveModel = user.into();

    user.tag_data = Set(serde_json::to_value(tag_data)?);

    user.update(ctx.db()).await?;

    Ok(())
}

pub async fn get_player_ghosts(
    ctx: &GatewayContext,
    persona_ids: Vec<i32>,
) -> Result<Vec<PlayerGhost>, GatewayError> {
    let users = Users::find()
        .filter(users::Column::PersonaId.is_in(persona_ids))
        .all(ctx.db())
        .await?;

    let ghosts = users
        .into_iter()
        .map(|user| {
            // TODO: fix weird syntax
            let variation = user.ghost_data["variation"]
                .as_i64()
                .unwrap_or(244578012)
                .to_string();
            let timestamp_val = user.ghost_data["timestamp"]
                .as_i64()
                .unwrap_or(0)
                .to_string();

            PlayerGhost {
                persona_id: user.persona_id.to_string(),
                ghost_data: GhostDataOutput {
                    customization: CustomizationOutput { variation },
                    timestamp: TimestampOutput {
                        timestamp_value: timestamp_val,
                    },
                },
            }
        })
        .collect();

    Ok(ghosts)
}

pub async fn get_latest_played(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<Vec<Entry>, GatewayError> {
    // Fetch challenge entries
    let challenge_entries_list = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .order_by_desc(challenge_entries::Column::CompletedAt)
        .limit(20)
        .all(ctx.db())
        .await?;

    // Fetch UGC entries
    let ugc_entries_list = ugc_entries::Entity::find()
        .filter(ugc_entries::Column::UserId.eq(persona_id))
        .order_by_desc(ugc_entries::Column::CompletedAt)
        .limit(20)
        .all(ctx.db())
        .await?;

    enum Fetched {
        Challenge(crate::entities::challenge_entries::Model),
        Ugc(crate::entities::ugc_entries::Model),
    }

    let mut combined: Vec<(chrono::DateTime<chrono::Utc>, Fetched)> = Vec::new();
    for entry in challenge_entries_list {
        combined.push((entry.completed_at, Fetched::Challenge(entry)));
    }
    for entry in ugc_entries_list {
        combined.push((entry.completed_at, Fetched::Ugc(entry)));
    }

    combined.sort_by(|a, b| b.0.cmp(&a.0));
    combined.truncate(20);

    let mut ugc_ids_to_fetch = Vec::new();
    for (_, fetched) in &combined {
        if let Fetched::Ugc(entry) = fetched {
            ugc_ids_to_fetch.push(entry.ugc_id);
        }
    }

    // Fetch UGC metadata for author IDs
    let ugc_models: std::collections::HashMap<uuid::Uuid, crate::entities::ugc::Model> =
        if !ugc_ids_to_fetch.is_empty() {
            ugc::Entity::find()
                .filter(ugc::Column::Id.is_in(ugc_ids_to_fetch))
                .all(ctx.db())
                .await?
                .into_iter()
                .map(|u| (u.id, u))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    let mut results = Vec::new();

    for (_, fetched) in combined {
        match fetched {
            Fetched::Challenge(entry) => {
                let user_stats = entry.user_stats;
                match entry.entry_type {
                    challenge_entries::ChallengeEntryType::HackableBillboard => {
                        if let Ok(stats) = serde_json::from_value(user_stats) {
                            results.push(Entry::Challenge(ChallengeEntry::HackableBillboard {
                                challenge_id: entry.challenge_id,
                                stats,
                            }));
                        }
                    }
                    challenge_entries::ChallengeEntryType::RunnersRoute => {
                        if let Ok(stats) = serde_json::from_value(user_stats) {
                            results.push(Entry::Challenge(ChallengeEntry::RunnersRoute {
                                challenge_id: entry.challenge_id,
                                stats,
                            }));
                        }
                    }
                }
            }
            Fetched::Ugc(entry) => {
                let user_stats = entry.user_stats;
                let author_id = ugc_models
                    .get(&entry.ugc_id)
                    .map(|u| u.author_id)
                    .unwrap_or(0);

                match entry.entry_type {
                    ugc_entries::UgcEntryType::ReachThis => {
                        if let Ok(stats) = serde_json::from_value(user_stats) {
                            results.push(Entry::Ugc(UgcEntry::ReachThis {
                                ugc_id: UgcId {
                                    user_id: author_id,
                                    id: entry.ugc_id.to_string(),
                                },
                                stats,
                            }));
                        }
                    }
                    ugc_entries::UgcEntryType::TimeTrial => {
                        if let Ok(stats) = serde_json::from_value(user_stats) {
                            results.push(Entry::Ugc(UgcEntry::TimeTrial {
                                ugc_id: UgcId {
                                    user_id: author_id,
                                    id: entry.ugc_id.to_string(),
                                },
                                stats,
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

pub async fn get_player_info(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<PlayerInfo, GatewayError> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or_else(|| GatewayError::internal("user not found"))?;

    let player_info = PlayerInfo {
        name: user.name.clone(),
        division: Division {
            name: user.division_name.clone(),
            rank: user.division_rank,
        },
        location: vec![],
    };

    Ok(player_info)
}

pub async fn get_player_ugc_limits(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<PlayerUgcLimits, GatewayError> {
    let counts: (i64, i64) = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .select_only()
        .column_as(ugc::Column::Id.count(), "total")
        .column_as(
            Expr::cust("COUNT(*) FILTER (WHERE published = true)"),
            "published",
        )
        .into_tuple::<(i64, i64)>()
        .one(ctx.db())
        .await?
        .unwrap_or((0, 0));

    Ok(PlayerUgcLimits {
        ugc_count: counts.0 as i32,
        max_ugc: 100,
        published_count: counts.1 as i32,
        max_published: 100,
    })
}
