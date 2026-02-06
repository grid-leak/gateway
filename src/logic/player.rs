use crate::context::GatewayContext;
use crate::entities::entries;
use crate::entities::{users, users::Entity as Users};
use crate::models::customization::{
    CustomizationOutput, GhostDataInput, GhostDataOutput, PlayerGhost, TagData, TimestampOutput,
};
use crate::models::game_data::{ChallengeEntry, Division, Entry, PlayerInfo, UgcEntry, UgcId};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};

pub async fn set_player_ghost(
    ctx: &GatewayContext,
    persona_id: i32,
    data: GhostDataInput,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or("User not found")?;

    let mut user: users::ActiveModel = user.into();

    user.ghost_variation = Set(data.customization.variation);

    // ignore the provided timestamp and set the current time
    let timestamp = Utc::now();

    user.ghost_timestamp = Set(timestamp);

    user.update(ctx.db()).await?;

    Ok(())
}

pub async fn set_player_tag(
    ctx: &GatewayContext,
    persona_id: i32,
    tag_data: TagData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or("User not found")?;

    let mut user: users::ActiveModel = user.into();

    user.tag_data = Set(serde_json::to_value(tag_data)?);

    user.update(ctx.db()).await?;

    Ok(())
}

pub async fn get_player_ghosts(
    ctx: &GatewayContext,
    persona_ids: Vec<i32>,
) -> Result<Vec<PlayerGhost>, Box<dyn std::error::Error + Send + Sync>> {
    let users = Users::find()
        .filter(users::Column::PersonaId.is_in(persona_ids))
        .all(ctx.db())
        .await?;

    let ghosts = users
        .into_iter()
        .map(|user| {
            let variation = user.ghost_variation.to_string();
            let timestamp_val = (user.ghost_timestamp.timestamp_millis() / 1000).to_string();

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
) -> Result<Vec<Entry>, Box<dyn std::error::Error + Send + Sync>> {
    let entries = entries::Entity::find()
        .filter(entries::Column::UserId.eq(persona_id))
        .limit(20)
        .all(ctx.db())
        .await?;

    let mut results = Vec::new();

    for entry in entries {
        let entry_type = entry.entry_type;
        let user_stats = entry.user_stats;

        match entry_type {
            entries::EntryType::ReachThis => {
                if let (Some(user_id), Some(id), Ok(stats)) = (
                    entry.ugc_author_id,
                    entry.ugc_id,
                    serde_json::from_value(user_stats),
                ) {
                    results.push(Entry::Ugc(UgcEntry::ReachThis {
                        ugc_id: UgcId {
                            user_id: user_id.to_string(),
                            id: id.to_string(),
                        },
                        stats,
                    }));
                }
            }
            entries::EntryType::TimeTrial => {
                if let (Some(user_id), Some(id), Ok(stats)) = (
                    entry.ugc_author_id,
                    entry.ugc_id,
                    serde_json::from_value(user_stats),
                ) {
                    results.push(Entry::Ugc(UgcEntry::TimeTrial {
                        ugc_id: UgcId {
                            user_id: user_id.to_string(),
                            id: id.to_string(),
                        },
                        stats,
                    }));
                }
            }
            entries::EntryType::HackableBillboard => {
                if let (Some(challenge_id), Ok(stats)) =
                    (entry.challenge_id, serde_json::from_value(user_stats))
                {
                    results.push(Entry::Challenge(ChallengeEntry::HackableBillboard {
                        challenge_id,
                        stats,
                    }));
                }
            }
            entries::EntryType::RunnersRoute => {
                if let (Some(challenge_id), Ok(stats)) =
                    (entry.challenge_id, serde_json::from_value(user_stats))
                {
                    results.push(Entry::Challenge(ChallengeEntry::RunnersRoute {
                        challenge_id,
                        stats,
                    }));
                }
            }
        }
    }

    Ok(results)
}

pub async fn get_player_info(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<PlayerInfo, Box<dyn std::error::Error + Send + Sync>> {
    let user = Users::find_by_id(persona_id)
        .one(ctx.db())
        .await?
        .ok_or("User not found")?;

    let player_info = PlayerInfo {
        name: user.name.clone(),
        division: Division {
            name: user.division_name.clone(),
            rank: user.division_rank,
        },
    };

    Ok(player_info)
}
