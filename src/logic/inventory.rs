use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    context::GatewayContext,
    entities::{user_items, user_kits},
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
        .into_iter()
        .map(|k| Kit {
            id: k.id.to_string().to_uppercase(),
            kit_type: k.kit_type.to_string().to_uppercase(),
            opened: k.opened,
        })
        .collect();

    let item_entries = user_items::Entity::find()
        .filter(user_items::Column::UserId.eq(persona_id))
        .all(db)
        .await
        .map_err(map_err)?;

    let items: Vec<Item> = item_entries
        .into_iter()
        .map(|i| Item {
            id: i.item_id,
            count: 1,
        })
        .collect();

    Ok(Inventory { kits, items })
}
