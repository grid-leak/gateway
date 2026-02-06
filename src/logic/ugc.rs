use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    context::GatewayContext,
    entities::{
        challenge_bookmarks,
        ugc::{self, UgcType},
        ugc_bookmarks, user_ugc_flags, users,
    },
    methods::map_err,
    models::game_data::{
        Bookmarks, ChallengeBookmarkEntry, Division, InitialGameDataResponse, LEVEL_ID_HASH,
        PlayerInfo, PromotedUgcWrapper, Transform, UgcBookmarkEntry, UgcId, UgcMeta, UgcWrapper,
    },
};

const UGC_BASE_URL: &str = "https://mec-gw.ops.dice.se/ugc/prod_default/prod_default/pc";

fn ugc_type_to_str(t: &UgcType) -> &'static str {
    match t {
        UgcType::ReachThis => "ReachThis",
        UgcType::TimeTrial => "TimeTrial",
    }
}

async fn build_ugc_meta(
    db: &sea_orm::DatabaseConnection,
    ugc_entry: &ugc::Model,
    author_name: &str,
    user_id: i32,
) -> Result<UgcMeta, jsonrpsee::types::ErrorObjectOwned> {
    let flags = user_ugc_flags::Entity::find()
        .filter(user_ugc_flags::Column::UserId.eq(user_id))
        .filter(user_ugc_flags::Column::UgcId.eq(ugc_entry.id))
        .one(db)
        .await
        .map_err(map_err)?;

    let (reported, blocked) = flags
        .map(|f| (f.reported, f.blocked))
        .unwrap_or((false, false));

    let level_id = LEVEL_ID_HASH;
    let type_id = ugc_type_to_str(&ugc_entry.r#type);

    let transform = Transform {
        x: ugc_entry.x,
        y: ugc_entry.y,
        z: ugc_entry.z,
        qx: Some(ugc_entry.qx),
        qy: Some(ugc_entry.qy),
        qz: Some(ugc_entry.qz),
        qw: Some(ugc_entry.qw),
    };

    // mapPosition for ReachThis, teleportTransform + ugcUrl for TimeTrial
    let (map_position, teleport_transform, ugc_url) = match ugc_entry.r#type {
        ugc::UgcType::ReachThis => {
            let pos = Transform {
                x: ugc_entry.x,
                y: ugc_entry.y,
                z: ugc_entry.z,
                qx: None,
                qy: None,
                qz: None,
                qw: None,
            };
            (Some(pos), None, None)
        }
        ugc::UgcType::TimeTrial => {
            let url = format!(
                "{}/{}/{}/{}",
                UGC_BASE_URL, type_id, ugc_entry.author_id, ugc_entry.id
            );
            (None, Some(transform.clone()), Some(url))
        }
    };

    Ok(UgcMeta {
        ugc_id: UgcId {
            user_id: ugc_entry.author_id.to_string(),
            id: ugc_entry.id.to_string(),
        },
        name: ugc_entry.name.clone(),
        creator_name: author_name.to_string(),
        created_at: ugc_entry.created_at.timestamp_millis().to_string(),
        updated_at: ugc_entry.updated_at.timestamp_millis().to_string(),
        published: ugc_entry.published,
        reported,
        blocked,
        level_id,
        transform,
        map_position,
        teleport_transform,
        ugc_url,
        type_id: type_id.to_string(),
    })
}

pub async fn get_initial_game_data(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
) -> Result<InitialGameDataResponse, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let user = users::Entity::find_by_id(persona_id)
        .one(db)
        .await
        .map_err(map_err)?
        .ok_or_else(|| map_err("User not found"))?;

    let player_info = PlayerInfo {
        name: user.name.clone(),
        division: Division {
            name: user.division_name.clone(),
            rank: user.division_rank,
        },
    };

    let user_stats = user.stats.clone();

    let reach_this_entries = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .filter(ugc::Column::Type.eq(ugc::UgcType::ReachThis))
        .all(db)
        .await
        .map_err(map_err)?;

    let mut user_reach_this = Vec::new();
    for entry in &reach_this_entries {
        let meta = build_ugc_meta(db, entry, &user.name, persona_id).await?;
        user_reach_this.push(UgcWrapper {
            meta,
            stats: None,
            user_stats: None,
            user_rank: None,
        });
    }

    let time_trial_entries = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .filter(ugc::Column::Type.eq(ugc::UgcType::TimeTrial))
        .all(db)
        .await
        .map_err(map_err)?;

    let mut user_time_trials = Vec::new();
    for entry in &time_trial_entries {
        let meta = build_ugc_meta(db, entry, &user.name, persona_id).await?;
        user_time_trials.push(UgcWrapper {
            meta,
            stats: None,
            user_stats: None,
            user_rank: None,
        });
    }

    let mut promoted_ugc = Vec::new();

    let new_ugc = ugc::Entity::find()
        .filter(ugc::Column::Published.eq(true))
        .order_by_desc(ugc::Column::CreatedAt)
        .all(db)
        .await
        .map_err(map_err)?;

    for entry in new_ugc.iter().take(2) {
        let author = users::Entity::find_by_id(entry.author_id)
            .one(db)
            .await
            .map_err(map_err)?;
        let author_name = author.map(|a| a.name).unwrap_or_default();
        let meta = build_ugc_meta(db, entry, &author_name, persona_id).await?;
        promoted_ugc.push(PromotedUgcWrapper {
            meta,
            reason: 2, // New
        });
    }

    let random_ugc = ugc::Entity::find()
        .filter(ugc::Column::Published.eq(true))
        .all(db)
        .await
        .map_err(map_err)?;

    let already_added: std::collections::HashSet<_> = promoted_ugc
        .iter()
        .map(|p| p.meta.ugc_id.id.clone())
        .collect();

    for entry in random_ugc
        .iter()
        .filter(|e| !already_added.contains(&e.id.to_string()))
        .take(2)
    {
        let author = users::Entity::find_by_id(entry.author_id)
            .one(db)
            .await
            .map_err(map_err)?;
        let author_name = author.map(|a| a.name).unwrap_or_default();
        let meta = build_ugc_meta(db, entry, &author_name, persona_id).await?;
        promoted_ugc.push(PromotedUgcWrapper {
            meta,
            reason: 1, // Random
        });
    }

    let ugc_bookmark_entries = ugc_bookmarks::Entity::find()
        .filter(ugc_bookmarks::Column::UserId.eq(persona_id))
        .all(db)
        .await
        .map_err(map_err)?;

    let mut ugc_bookmarks_list = Vec::new();
    for bookmark in &ugc_bookmark_entries {
        if let Some(ugc_entry) = ugc::Entity::find_by_id(bookmark.ugc_id)
            .one(db)
            .await
            .map_err(map_err)?
        {
            let author = users::Entity::find_by_id(ugc_entry.author_id)
                .one(db)
                .await
                .map_err(map_err)?;
            let author_name = author.map(|a| a.name).unwrap_or_default();
            let meta = build_ugc_meta(db, &ugc_entry, &author_name, persona_id).await?;
            ugc_bookmarks_list.push(UgcBookmarkEntry {
                ugc_type: ugc_type_to_str(&ugc_entry.r#type).to_string(),
                bookmark_time: bookmark.bookmark_time.to_string(),
                meta,
            });
        }
    }

    let challenge_bookmark_entries = challenge_bookmarks::Entity::find()
        .filter(challenge_bookmarks::Column::UserId.eq(persona_id))
        .all(db)
        .await
        .map_err(map_err)?;

    let challenge_bookmarks_list: Vec<ChallengeBookmarkEntry> = challenge_bookmark_entries
        .into_iter()
        .map(|b| ChallengeBookmarkEntry {
            challenge_id: b.challenge_id,
            bookmark_time: b.bookmark_time.to_string(),
            challenge_type: b.challenge_type,
        })
        .collect();

    let inventory = super::inventory::get_inventory(ctx, persona_id).await?;

    Ok(InitialGameDataResponse {
        player_info,
        user_stats,
        user_reach_this,
        user_time_trials,
        promoted_ugc,
        bookmarks: Bookmarks {
            ugc_bookmarks: ugc_bookmarks_list,
            challenge_bookmarks: challenge_bookmarks_list,
        },
        inventory,
    })
}

pub async fn get_player_info(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
) -> Result<PlayerInfo, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let user = users::Entity::find_by_id(persona_id)
        .one(db)
        .await
        .map_err(map_err)?
        .ok_or_else(|| map_err("User not found"))?;

    let player_info = PlayerInfo {
        name: user.name.clone(),
        division: Division {
            name: user.division_name.clone(),
            rank: user.division_rank,
        },
    };

    Ok(player_info)
}

// TODO: get_latest_played