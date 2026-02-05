use std::collections::HashSet;
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::{
    context::GatewayContext,
    entities::user_kits,
    logic::kit_data,
    methods::map_err,
    models::game_data::{Inventory, Item, Kit},
};

pub async fn get_inventory(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
) -> Result<Inventory, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let kit_entries = user_kits::Entity::find()
        .filter(user_kits::Column::UserId.eq(persona_id))
        .all(db)
        .await
        .map_err(map_err)?;

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
) -> Result<Kit, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id).map_err(|e| {
        jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            format!("Invalid kit ID: {}", e),
            None::<()>,
        )
    })?;

    let kit_type_str = kit_data::get_kit_type(kit_id).ok_or_else(|| {
        jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Unknown kit ID", None::<()>)
    })?;

    let kit_type_uuid = Uuid::parse_str(kit_type_str).map_err(|e| {
        jsonrpsee::types::ErrorObjectOwned::owned(
            -32603,
            format!("Invalid kit type UUID: {}", e),
            None::<()>,
        )
    })?;

    let new_kit = user_kits::ActiveModel {
        user_id: Set(persona_id),
        kit_id: Set(kit_uuid),
        kit_type: Set(kit_type_uuid),
        opened: Set(false),
        ..Default::default()
    };

    new_kit.insert(db).await.map_err(map_err)?;

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
) -> Result<Vec<Item>, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id).map_err(|e| {
        jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            format!("Invalid kit ID: {}", e),
            None::<()>,
        )
    })?;

    let kit_entry = user_kits::Entity::find()
        .filter(user_kits::Column::UserId.eq(persona_id))
        .filter(user_kits::Column::KitId.eq(kit_uuid))
        .one(db)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Kit not found", None::<()>)
        })?;

    if kit_entry.opened {
        return Err(jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            "Kit already opened",
            None::<()>,
        ));
    }

    let mut active_kit: user_kits::ActiveModel = kit_entry.into();
    active_kit.opened = Set(true);
    active_kit.update(db).await.map_err(map_err)?;

    let rewards = kit_data::get_kit_rewards(kit_id).ok_or_else(|| {
        jsonrpsee::types::ErrorObjectOwned::owned(-32603, "Kit rewards not found", None::<()>)
    })?;

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
) -> Result<Vec<Item>, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let kit_uuid = Uuid::parse_str(kit_id).map_err(|e| {
        jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            format!("Invalid kit ID: {}", e),
            None::<()>,
        )
    })?;

    let result = user_kits::Entity::delete_many()
        .filter(user_kits::Column::UserId.eq(persona_id))
        .filter(user_kits::Column::KitId.eq(kit_uuid))
        .exec(db)
        .await
        .map_err(map_err)?;

    if result.rows_affected == 0 {
        return Err(jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            "Kit not found",
            None::<()>,
        ));
    }

    // Game expects the same response body as the openKit
    // But I haven't actually recorded a `revokeKit`, so
    // I will have to assume it wants to return count 0

    let rewards = kit_data::get_kit_rewards(kit_id).ok_or_else(|| {
        jsonrpsee::types::ErrorObjectOwned::owned(-32603, "Kit rewards not found", None::<()>)
    })?;

    let items: Vec<Item> = rewards
        .iter()
        .map(|id| Item {
            id: (*id).to_string(),
            count: 0,
        })
        .collect();

    Ok(items)
}
