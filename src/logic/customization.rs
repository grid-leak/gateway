use crate::context::GatewayContext;
use crate::entities::{users, users::Entity as Users};
use crate::models::customization::{
    CustomizationOutput, GhostDataInput, GhostDataOutput, PlayerGhost, TagData, TimestampOutput,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

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
