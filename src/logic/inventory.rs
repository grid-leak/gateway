use std::collections::{HashMap, HashSet};

use sea_orm::{ActiveModelTrait, EntityTrait, QuerySelect, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    context::GatewayContext,
    logic::{GatewayError, kit_data},
    models::game_data::{Inventory, Item, Kit},
};
use entities::user_kits;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitStatus {
    pub kit_type: Uuid,
    pub opened: bool,
}

type KitsMap = HashMap<Uuid, KitStatus>;

pub async fn get_inventory(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<Inventory, GatewayError> {
    let db = ctx.db();

    let user_kit_entry = user_kits::Entity::find_by_id(persona_id).one(db).await?;

    let kits_map: KitsMap = match user_kit_entry {
        Some(entry) => serde_json::from_value(entry.kits).map_err(|e| {
            tracing::error!(persona_id, error = ?e, "Failed to deserialize user kits JSON");
            GatewayError::internal("corrupted user inventory data")
        })?,
        None => KitsMap::new(),
    };

    let kits: Vec<Kit> = kits_map
        .iter()
        .map(|(kit_uuid, status)| Kit {
            id: kit_uuid.to_string().to_uppercase(),
            kit_type: status.kit_type.to_string().to_uppercase(),
            opened: status.opened,
        })
        .collect();

    // Deriving items based on user kits
    let mut item_ids: HashSet<&str> = kit_data::get_default_items().iter().copied().collect();

    for (kit_uuid, status) in &kits_map {
        if status.opened {
            if let Some(rewards) = kit_data::get_kit_rewards(&kit_uuid.to_string()) {
                item_ids.extend(rewards.iter().copied());
            } else {
                tracing::warn!(?kit_uuid, "Rewards definition missing for opened kit");
            }
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
    ctx: &GatewayContext,
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

    let kit = db.transaction::<_, Kit, GatewayError>(|txn| {
        Box::pin(async move {
            let entry_opt = user_kits::Entity::find_by_id(persona_id)
                .lock_exclusive()
                .one(txn)
                .await?;

            let mut kits_map: KitsMap = match &entry_opt {
                Some(entry) => serde_json::from_value(entry.kits.clone())
                    .map_err(|e| {
                        tracing::error!(persona_id, error = ?e, "Failed to deserialize user kits JSON");
                        GatewayError::internal("corrupted user inventory data")
                    })?,
                None => KitsMap::new(),
            };

            if kits_map.contains_key(&kit_uuid) {
                return Err(GatewayError::invalid_params("kit already granted"));
            }

            kits_map.insert(kit_uuid, KitStatus {
                kit_type: kit_type_uuid,
                opened: false,
            });

            let kits_json = serde_json::to_value(kits_map)
                .map_err(|e| GatewayError::internal(format!("failed to serialize kits: {e}")))?;

            match entry_opt {
                Some(entry) => {
                    let mut active: user_kits::ActiveModel = entry.into();
                    active.kits = Set(kits_json);
                    active.update(txn).await?;
                }
                None => {
                    let active = user_kits::ActiveModel {
                        user_id: Set(persona_id),
                        kits: Set(kits_json),
                    };
                    active.insert(txn).await?;
                }
            }

            Ok(Kit {
                id: kit_uuid.to_string().to_uppercase(),
                kit_type: kit_type_str.to_uppercase(),
                opened: false,
            })
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Connection(db_err) => GatewayError::from(db_err),
        sea_orm::TransactionError::Transaction(gw_err) => gw_err,
    })?;

    Ok(kit)
}

pub async fn open_kit(
    ctx: &GatewayContext,
    persona_id: i32,
    kit_id: &str,
) -> Result<Vec<Item>, GatewayError> {
    let db = ctx.db();

    let rewards = kit_data::get_kit_rewards(kit_id)
        .ok_or_else(|| GatewayError::internal("kit rewards not found"))?;

    let kit_uuid = Uuid::parse_str(kit_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid kit ID: {e}")))?;

    let items = db
        .transaction::<_, Vec<Item>, GatewayError>(|txn| {
            Box::pin(async move {
                let entry = user_kits::Entity::find_by_id(persona_id)
                    .lock_exclusive()
                    .one(txn)
                    .await?
                    .ok_or_else(|| GatewayError::invalid_params("inventory not found"))?;

                let mut kits_map: KitsMap = serde_json::from_value(entry.kits.clone())
                .map_err(|e| {
                    tracing::error!(persona_id, error = ?e, "Failed to deserialize user kits JSON");
                    GatewayError::internal("corrupted user inventory data")
                })?;

                let status = kits_map
                    .get_mut(&kit_uuid)
                    .ok_or_else(|| GatewayError::invalid_params("kit not found"))?;

                if status.opened {
                    return Err(GatewayError::invalid_params("kit already opened"));
                }

                status.opened = true;

                let kits_json = serde_json::to_value(kits_map).map_err(|e| {
                    GatewayError::internal(format!("failed to serialize kits: {e}"))
                })?;

                let mut active: user_kits::ActiveModel = entry.into();
                active.kits = Set(kits_json);
                active.update(txn).await?;

                let items: Vec<Item> = rewards
                    .iter()
                    .map(|id| Item {
                        id: (*id).to_string(),
                        count: 1,
                    })
                    .collect();

                Ok(items)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => GatewayError::from(db_err),
            sea_orm::TransactionError::Transaction(gw_err) => gw_err,
        })?;

    Ok(items)
}

pub async fn revoke_kit(
    ctx: &GatewayContext,
    persona_id: i32,
    kit_id: &str,
) -> Result<Vec<Item>, GatewayError> {
    let db = ctx.db();

    let rewards = kit_data::get_kit_rewards(kit_id)
        .ok_or_else(|| GatewayError::internal("kit rewards not found"))?;

    let kit_uuid = Uuid::parse_str(kit_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid kit ID: {e}")))?;

    let items = db
        .transaction::<_, Vec<Item>, GatewayError>(|txn| {
            Box::pin(async move {
                let entry = user_kits::Entity::find_by_id(persona_id)
                    .lock_exclusive()
                    .one(txn)
                    .await?
                    .ok_or_else(|| GatewayError::invalid_params("inventory not found"))?;

                let mut kits_map: KitsMap = serde_json::from_value(entry.kits.clone())
                .map_err(|e| {
                    tracing::error!(persona_id, error = ?e, "Failed to deserialize user kits JSON");
                    GatewayError::internal("corrupted user inventory data")
                })?;

                if kits_map.remove(&kit_uuid).is_none() {
                    return Err(GatewayError::invalid_params("kit not found"));
                }

                let kits_json = serde_json::to_value(kits_map).map_err(|e| {
                    GatewayError::internal(format!("failed to serialize kits: {e}"))
                })?;

                let mut active: user_kits::ActiveModel = entry.into();
                active.kits = Set(kits_json);
                active.update(txn).await?;

                let items: Vec<Item> = rewards
                    .iter()
                    .map(|id| Item {
                        id: (*id).to_string(),
                        count: 0,
                    })
                    .collect();

                Ok(items)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => GatewayError::from(db_err),
            sea_orm::TransactionError::Transaction(gw_err) => gw_err,
        })?;

    Ok(items)
}
