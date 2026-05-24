use crate::{
    context::GatewayContext,
    logic::{
        GatewayError,
        game_data::{load_ugc_flags, ugc_to_meta, ugc_type_to_string},
    },
    models::game_data::{Bookmarks, ChallengeBookmarkEntry, UgcBookmarkEntry},
};
use chrono::Utc;
use entities::{challenge_bookmarks, ugc, ugc_bookmarks, users};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set, sea_query::OnConflict};
use uuid::Uuid;

pub async fn get_bookmarks(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<Bookmarks, GatewayError> {
    let db = ctx.db();
    let user = ctx.user(persona_id).await?;

    let (ugc_bm_data, challenge_bm_data) = tokio::try_join!(
        async {
            user.find_related(ugc_bookmarks::Entity)
                .find_also_related(ugc::Entity)
                .all(db)
                .await
                .map_err(GatewayError::from)
        },
        async {
            user.find_related(challenge_bookmarks::Entity)
                .all(db)
                .await
                .map_err(GatewayError::from)
        }
    )?;

    // Collect UGC IDs and author IDs from the already-joined UGC models
    let valid_ugcs: Vec<&ugc::Model> = ugc_bm_data
        .iter()
        .filter_map(|(_, ugc_opt)| ugc_opt.as_ref())
        .collect();

    let ugc_ids: Vec<uuid::Uuid> = valid_ugcs.iter().map(|u| u.id).collect();
    let author_ids: Vec<i32> = valid_ugcs
        .iter()
        .map(|u| u.author_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let (flags_map, authors_map) =
        tokio::try_join!(load_ugc_flags(db, persona_id, &ugc_ids), async {
            if author_ids.is_empty() {
                return Ok(std::collections::HashMap::new());
            }
            users::Entity::find()
                .filter(users::Column::PersonaId.is_in(author_ids))
                .all(db)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|u| (u.persona_id, u.name))
                        .collect::<std::collections::HashMap<i32, String>>()
                })
                .map_err(GatewayError::from)
        })?;

    let ugc_bookmarks_list: Vec<UgcBookmarkEntry> = ugc_bm_data
        .into_iter()
        .filter_map(|(bm, ugc_opt)| {
            let entry = ugc_opt?;
            let author = authors_map
                .get(&entry.author_id)
                .map(|s| s.as_str())
                .unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            Some(UgcBookmarkEntry {
                ugc_type: ugc_type_to_string(&entry.r#type),
                bookmark_time: bm.bookmark_time.timestamp_millis().to_string(),
                meta: ugc_to_meta(entry, author, &flags),
            })
        })
        .collect();

    let challenge_bookmarks_list: Vec<ChallengeBookmarkEntry> = challenge_bm_data
        .into_iter()
        .map(|b| ChallengeBookmarkEntry {
            challenge_id: b.challenge_id,
            bookmark_time: b.bookmark_time.timestamp_millis().to_string(),
            challenge_type: b.challenge_type,
        })
        .collect();

    Ok(Bookmarks {
        ugc_bookmarks: ugc_bookmarks_list,
        challenge_bookmarks: challenge_bookmarks_list,
    })
}

pub async fn add_ugc_bookmark(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: Uuid,
) -> Result<(), GatewayError> {
    let now = Utc::now();

    ugc_bookmarks::Entity::insert(ugc_bookmarks::ActiveModel {
        user_id: Set(persona_id),
        ugc_id: Set(ugc_id),
        bookmark_time: Set(now),
    })
    .on_conflict(
        OnConflict::columns([ugc_bookmarks::Column::UserId, ugc_bookmarks::Column::UgcId])
            .update_column(ugc_bookmarks::Column::BookmarkTime)
            .to_owned(),
    )
    .exec(ctx.db())
    .await?;

    Ok(())
}

pub async fn remove_ugc_bookmark(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: Uuid,
) -> Result<(), GatewayError> {
    ugc_bookmarks::Entity::delete_many()
        .filter(ugc_bookmarks::Column::UserId.eq(persona_id))
        .filter(ugc_bookmarks::Column::UgcId.eq(ugc_id))
        .exec(ctx.db())
        .await?;

    Ok(())
}

pub async fn add_challenge_bookmark(
    ctx: &GatewayContext,
    persona_id: i32,
    challenge_id: String,
    challenge_type: String,
) -> Result<(), GatewayError> {
    let now = Utc::now();

    challenge_bookmarks::Entity::insert(challenge_bookmarks::ActiveModel {
        user_id: Set(persona_id),
        challenge_id: Set(challenge_id),
        challenge_type: Set(challenge_type),
        bookmark_time: Set(now),
    })
    .on_conflict(
        OnConflict::columns([
            challenge_bookmarks::Column::UserId,
            challenge_bookmarks::Column::ChallengeId,
        ])
        .update_column(challenge_bookmarks::Column::BookmarkTime)
        .to_owned(),
    )
    .exec(ctx.db())
    .await?;

    Ok(())
}

pub async fn remove_challenge_bookmark(
    ctx: &GatewayContext,
    persona_id: i32,
    challenge_id: String,
) -> Result<(), GatewayError> {
    challenge_bookmarks::Entity::delete_many()
        .filter(challenge_bookmarks::Column::UserId.eq(persona_id))
        .filter(challenge_bookmarks::Column::ChallengeId.eq(challenge_id))
        .exec(ctx.db())
        .await?;

    Ok(())
}
