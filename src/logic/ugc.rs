use std::{collections::HashMap, str::FromStr, sync::Arc};

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter,
    QueryOrder, QuerySelect, Set,
    sea_query::{Alias, Expr, JoinType, PostgresQueryBuilder, Query},
};
use uuid::Uuid;

use crate::{
    context::GatewayContext,
    entities::{
        challenge_bookmarks,
        entries::{self, EntryType},
        ugc::{self, UgcType},
        ugc_bookmarks, user_ugc_flags, users,
    },
    methods::map_err,
    models::{
        game_data::{
            Bookmarks, ChallengeBookmarkEntry, Division, InitialGameDataResponse, LEVEL_ID_HASH,
            PlayerInfo, PromotedUgcWrapper, ReachThisWrapper, Transform, UgcBookmarkEntry, UgcId,
            UgcMeta, UgcWrapper, UserRank,
        },
        ugc::CreateReachThisMeta,
        user_stats::ReachThisUserStats,
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
        .limit(2)
        .all(db)
        .await
        .map_err(map_err)?;

    for entry in new_ugc.iter() {
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
        .limit(2)
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

pub async fn create_reach_this(
    ctx: &GatewayContext,
    author_id: i32,
    reach_this: CreateReachThisMeta,
) -> Result<UgcMeta, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let limits = super::player::get_player_ugc_limits(ctx, author_id)
        .await
        .map_err(map_err)?;

    // TODO: return proper error codes for the game to handle them
    if limits.ugc_count >= limits.max_ugc {
        return Err(jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            "UGC creation limit reached",
            None::<()>,
        ));
    }
    if reach_this.published && limits.published_count >= limits.max_published {
        return Err(jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            "UGC publish limit reached",
            None::<()>,
        ));
    }

    let user = users::Entity::find_by_id(author_id)
        .one(db)
        .await
        .map_err(map_err)?
        .ok_or_else(|| map_err("User not found"))?;

    let now = Utc::now();
    let new_id = Uuid::new_v4();
    let transform = reach_this.transform;

    let new_ugc = ugc::ActiveModel {
        id: Set(new_id),
        author_id: Set(author_id),
        name: Set(reach_this.name),
        r#type: Set(UgcType::ReachThis),
        created_at: Set(now),
        updated_at: Set(now),
        published: Set(reach_this.published),
        x: Set(transform.x),
        y: Set(transform.y),
        z: Set(transform.z),
        qx: Set(transform.qx.unwrap_or(0.0)),
        qy: Set(transform.qy.unwrap_or(0.0)),
        qz: Set(transform.qz.unwrap_or(0.0)),
        qw: Set(transform.qw.unwrap_or(1.0)),
    };

    let ugc_model: ugc::Model = new_ugc.insert(db).await.map_err(map_err)?;

    build_ugc_meta(db, &ugc_model, &user.name, author_id).await
}

// TODO: update existing entry
pub async fn finish_reach_this(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
    ugc_author_id: i32,
) -> Result<ReachThisWrapper, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();
    let now = chrono::Utc::now();

    entries::Entity::insert(entries::ActiveModel {
        user_id: Set(persona_id),
        ugc_id: Set(Some(Uuid::from_str(&ugc_id).unwrap())),
        ugc_author_id: Set(Some(ugc_author_id)),
        challenge_id: Set(None),
        entry_type: Set(EntryType::ReachThis),
        completed_at: Set(now),
        user_stats: Set(serde_json::Value::Null),
        score: Set(0),
        ..Default::default()
    })
    .exec(db)
    .await
    .map_err(map_err)?;

    Ok(ReachThisWrapper {
        meta: None,
        stats: None,
        user_stats: Some(ReachThisUserStats {
            reached_at: now.timestamp_millis().to_string(),
        }),
        user_rank: Some(UserRank {
            rank: 1,
            score: now.timestamp_millis().to_string(),
            total: 1,
        }),
    })
}

#[derive(Debug, FromQueryResult)]
struct CountResult {
    ugc_id: Option<Uuid>,
    count: i64,
}

pub async fn get_reach_this_data(
    ctx: &Arc<GatewayContext>,
    ugc_ids: Vec<String>,
    data_types: Vec<String>,
    persona_id: i32,
) -> Result<Vec<ReachThisWrapper>, jsonrpsee::types::ErrorObjectOwned> {
    if ugc_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db = ctx.db();
    let mut responses = Vec::with_capacity(ugc_ids.len());
    let requested_ids: Vec<Uuid> = ugc_ids
        .iter()
        .filter_map(|u| Uuid::from_str(&u).ok())
        .collect();

    // 1. Fetch User Stats & Ranks if requested
    let mut user_stats_map = HashMap::new();
    let mut user_ranks_map = HashMap::new();
    let mut totals_map = HashMap::new();

    if data_types.contains(&"USER_STATS".to_string()) {
        // Fetch user entries
        let user_entries = entries::Entity::find()
            .filter(entries::Column::UserId.eq(persona_id))
            .filter(entries::Column::UgcId.is_in(requested_ids.clone()))
            .filter(entries::Column::EntryType.eq(EntryType::ReachThis))
            .all(db)
            .await
            .map_err(map_err)?;

        for entry in user_entries {
            if let Some(uid) = entry.ugc_id {
                user_stats_map.insert(uid, entry);
            }
        }

        // Fetch totals
        let totals: Vec<CountResult> = entries::Entity::find()
            .select_only()
            .column(entries::Column::UgcId)
            .column_as(entries::Column::Id.count(), "count")
            .filter(entries::Column::UgcId.is_in(requested_ids.clone()))
            .filter(entries::Column::EntryType.eq(EntryType::ReachThis))
            .group_by(entries::Column::UgcId)
            .into_model::<CountResult>()
            .all(db)
            .await
            .map_err(map_err)?;

        for t in totals {
            if let Some(uid) = t.ugc_id {
                totals_map.insert(uid, t.count);
            }
        }

        // Fetch Ranks (Completed At Ascending - earlier is better)
        if !user_stats_map.is_empty() {
            let t1 = Alias::new("t1");
            let t2 = Alias::new("t2");

            let t1_id = (t1.clone(), entries::Column::UgcId);
            let t2_id = (t2.clone(), entries::Column::UgcId);
            let t1_score = (t1.clone(), entries::Column::CompletedAt);
            let t2_score = (t2.clone(), entries::Column::CompletedAt);

            // Count how many people have completed_at < my completed_at
            let join_condition = Expr::col(t1_id.clone())
                .equals(t2_id.clone())
                .and(Expr::col(t2_score).lt(Expr::col(t1_score)));

            let mut query = Query::select();
            query
                .column(t1_id.clone())
                .expr_as(
                    Expr::col((t2.clone(), entries::Column::Id)).count(),
                    "count",
                )
                .from_as(entries::Entity, t1.clone())
                .join_as(
                    JoinType::LeftJoin,
                    entries::Entity,
                    t2.clone(),
                    join_condition,
                )
                .and_where(Expr::col((t1.clone(), entries::Column::UserId)).eq(persona_id))
                .and_where(Expr::col(t1_id.clone()).is_in(requested_ids.clone()))
                .and_where(
                    Expr::col((t1.clone(), entries::Column::EntryType)).eq(EntryType::ReachThis),
                )
                .group_by_col(t1_id);

            let (sql, values) = query.build(PostgresQueryBuilder);

            let rank_results = CountResult::find_by_statement(
                sea_orm::Statement::from_sql_and_values(DbBackend::Postgres, &sql, values),
            )
            .all(db)
            .await
            .map_err(map_err)?;

            for r in rank_results {
                if let Some(uid) = r.ugc_id {
                    user_ranks_map.insert(uid, r.count);
                }
            }
        }
    }

    // 2. Fetch Meta if requested
    let mut meta_map = HashMap::new();
    if data_types.contains(&"META".to_string()) {
        let ugc_entries = ugc::Entity::find()
            .filter(ugc::Column::Id.is_in(requested_ids.clone()))
            .all(db)
            .await
            .map_err(map_err)?;

        for entry in ugc_entries {
            // Need author name
            let author = users::Entity::find_by_id(entry.author_id)
                .one(db)
                .await
                .map_err(map_err)?;
            let author_name = author.map(|a| a.name).unwrap_or_default();

            let meta = build_ugc_meta(db, &entry, &author_name, persona_id).await?;
            meta_map.insert(entry.id, meta);
        }
    }

    // 3. Construct Response
    for ugc_id in ugc_ids {
        let uid = Uuid::from_str(&ugc_id).unwrap_or_default();

        let meta = meta_map.remove(&uid);

        let mut user_stats = None;
        let mut user_rank = None;

        if let Some(entry) = user_stats_map.get(&uid) {
            user_stats = Some(ReachThisUserStats {
                reached_at: entry.completed_at.timestamp_millis().to_string(),
            });

            let better_count = *user_ranks_map.get(&uid).unwrap_or(&0);
            let total_entries = *totals_map.get(&uid).unwrap_or(&0);

            user_rank = Some(UserRank {
                rank: (better_count + 1) as i32,
                score: entry.completed_at.timestamp_millis().to_string(),
                total: total_entries,
            });
        }

        responses.push(ReachThisWrapper {
            meta,
            stats: None,
            user_stats,
            user_rank,
        });
    }

    Ok(responses)
}

pub async fn get_overview_reach_this_leaderboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    ugc_uuid: String,
    radius: Option<i32>,
) -> Result<
    crate::models::game_data::OverviewReachThisLeaderboardResponse,
    jsonrpsee::types::ErrorObjectOwned,
> {
    let db = ctx.db();
    let ugc_uuid = Uuid::from_str(&ugc_uuid).map_err(map_err)?;
    let radius = std::cmp::max(radius.unwrap_or(3), 0);

    // 1. Fetch all entries for this UGC, sorted by completion time (asc)
    // We need to fetch enough to determine ranks.
    // Ideally we'd use window functions (RANK() OVER ...), but SeaORM raw query or fetching all is needed.
    // Given potentially large number of entries, fetched all might be heavy.
    // Optimization: Count total, and fetch window around user or top.

    // For now, let's fetch all IDs and their completion times to sort in memory or DB.
    // DB sort is better.

    // We need: (user_id, completed_at, rank)

    let all_entries = entries::Entity::find()
        .filter(entries::Column::UgcId.eq(ugc_uuid))
        .filter(entries::Column::EntryType.eq(EntryType::ReachThis))
        .order_by_asc(entries::Column::CompletedAt)
        .all(db)
        .await
        .map_err(map_err)?;

    let total_count = all_entries.len() as i64;

    // Find my rank
    let my_rank_index = all_entries.iter().position(|e| e.user_id == persona_id);

    let center_rank = if let Some(idx) = my_rank_index {
        idx as i32
    } else {
        0 // Default to top if not found
    };

    let start_rank = std::cmp::max(center_rank - radius, 0);
    let end_rank = std::cmp::min(center_rank + radius, (total_count - 1) as i32);

    // Re-adjust if user not found, per request "if there's no user entry just show the top scores"
    // My previous logic: if index is None (user not found), center_rank is 0.
    // start_rank = (0 - 3).max(0) = 0.
    // end_rank = (0 + 3).min(max) = 3.
    // So it returns 0..3 (top 4). That matches behavior "show top scores".
    // If I am at rank 100, radius 3. start=97, end=103.

    let slice = if total_count > 0 {
        &all_entries[start_rank as usize..=end_rank as usize]
    } else {
        &[]
    };

    let mut users_list = Vec::new();

    for (i, entry) in slice.iter().enumerate() {
        let rank = start_rank + i as i32 + 1; // 1-based rank

        let user = users::Entity::find_by_id(entry.user_id)
            .one(db)
            .await
            .map_err(map_err)?
            .ok_or_else(|| map_err("User not found"))?;

        users_list.push(crate::models::game_data::LeaderboardUser {
            position: rank,
            global_rank: rank,
            score: entry.completed_at.timestamp_millis().to_string(),
            percentile: None,
            persona_id: user.persona_id.to_string(),
            name: user.name,
            division: Division {
                name: user.division_name,
                rank: user.division_rank,
            },
        });
    }

    // Global Leader
    let global_leader = if total_count > 0 {
        let entry = &all_entries[0];
        let user = users::Entity::find_by_id(entry.user_id)
            .one(db)
            .await
            .map_err(map_err)?
            .ok_or_else(|| map_err("User not found"))?;

        Some(crate::models::game_data::LeaderboardUser {
            position: 1,
            global_rank: 1,
            score: entry.completed_at.timestamp_millis().to_string(),
            percentile: None,
            persona_id: user.persona_id.to_string(),
            name: user.name,
            division: Division {
                name: user.division_name,
                rank: user.division_rank,
            },
        })
    } else {
        None
    };

    Ok(
        crate::models::game_data::OverviewReachThisLeaderboardResponse {
            leaderboard: crate::models::game_data::LeaderboardWrapper {
                area: None,
                total_count,
                users: users_list,
            },
            global_leader,
        },
    )
}
