use std::collections::HashSet;
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::{
    context::GatewayContext,
    entities::user_kits,
    logic::{GatewayError, kit_data},
    models::game_data::{Inventory, Item, Kit},
};

pub async fn get_inventory(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
) -> Result<Inventory, GatewayError> {
    let db = ctx.db();

    let user = ctx.user(persona_id).await?;
    let kit_entries = user.find_related(user_kits::Entity).all(db).await?;

    let kits: Vec<Kit> = kit_entries
        .iter()
        .map(|k| Kit {
            id: k.kit_id.to_string().to_uppercase(),
            kit_type: k.kit_type.to_string().to_uppercase(),
            opened: k.opened,
        })
        .collect();

    // Deriving items based on user kits
    // Start with default items
    let mut item_ids: HashSet<&str> = kit_data::get_default_items().iter().copied().collect();

    // Add rewards from all opened kits
    for kit_entry in &kit_entries {
        if kit_entry.opened
            && let Some(rewards) = kit_data::get_kit_rewards(&kit_entry.kit_id.to_string())
        {
            item_ids.extend(rewards.iter().copied());
        }
    }

    let items: Vec<Item> = item_ids
        .into_iter()
        .map(|id| Item {
            id: id.to_string(),
            count: 1,
        })
        .collect();

    Ok(Inventory { kits, items })
}

pub async fn grant_kit(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    kit_id: &str,
) -> Result<Kit, GatewayError> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid kit ID: {e}")))?;

    let kit_type_str = kit_data::get_kit_type(kit_id)
        .ok_or_else(|| GatewayError::invalid_params("unknown kit ID"))?;

    let kit_type_uuid = Uuid::parse_str(kit_type_str)
        .map_err(|e| GatewayError::internal(format!("invalid kit type UUID: {e}")))?;

    let new_kit = user_kits::ActiveModel {
        user_id: Set(persona_id),
        kit_id: Set(kit_uuid),
        kit_type: Set(kit_type_uuid),
        opened: Set(false),
        ..Default::default()
    };

    new_kit.insert(db).await?;

    Ok(Kit {
        id: kit_uuid.to_string().to_uppercase(),
        kit_type: kit_type_str.to_uppercase(),
        opened: false,
    })
}

pub async fn open_kit(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    kit_id: &str,
) -> Result<Vec<Item>, GatewayError> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid kit ID: {e}")))?;

    let kit_entry = user_kits::Entity::find()
        .filter(user_kits::Column::UserId.eq(persona_id))
        .filter(user_kits::Column::KitId.eq(kit_uuid))
        .one(db)
        .await?
        .ok_or_else(|| GatewayError::invalid_params("kit not found"))?;

    if kit_entry.opened {
        return Err(GatewayError::invalid_params("kit already opened"));
    }

    let mut active_kit: user_kits::ActiveModel = kit_entry.into();
    active_kit.opened = Set(true);
    active_kit.update(db).await?;

    let rewards = kit_data::get_kit_rewards(kit_id)
        .ok_or_else(|| GatewayError::internal("kit rewards not found"))?;

    let items: Vec<Item> = rewards
        .iter()
        .map(|id| Item {
            id: (*id).to_string(),
            count: 1,
        })
        .collect();

    Ok(items)
}

pub async fn revoke_kit(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    kit_id: &str,
) -> Result<Vec<Item>, GatewayError> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid kit ID: {e}")))?;

    let result = user_kits::Entity::delete_many()
        .filter(user_kits::Column::UserId.eq(persona_id))
        .filter(user_kits::Column::KitId.eq(kit_uuid))
        .exec(db)
        .await?;

    if result.rows_affected == 0 {
        return Err(GatewayError::invalid_params("kit not found"));
    }

    // Game expects the same response body as the openKit
    // But I haven't actually recorded a `revokeKit`, so
    // I will have to assume it wants to return count 0

    let rewards = kit_data::get_kit_rewards(kit_id)
        .ok_or_else(|| GatewayError::internal("kit rewards not found"))?;

    let items: Vec<Item> = rewards
        .iter()
        .map(|id| Item {
            id: (*id).to_string(),
            count: 0,
        })
        .collect();

    Ok(items)
}
