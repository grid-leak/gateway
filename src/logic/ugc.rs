use crate::{
    context::GatewayContext,
    entities::{
        challenge_bookmarks,
        entries::{self, EntryType},
        ugc::{self, UgcType},
        ugc_bookmarks, users,
    },
    logic::game_data::{BatchUgcLoader, UgcFlags},
    methods::map_err,
    models::{
        game_data::{
            Bookmarks, ChallengeBookmarkEntry, Division, InitialGameDataResponse, LEVEL_ID_HASH,
            PlayerInfo, PromotedUgcWrapper, ReachThisWrapper, UgcBookmarkEntry, UgcMeta,
            UgcWrapper, UserRank,
        },
        ugc::CreateReachThisMeta,
        user_stats::ReachThisUserStats,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter,
    QueryOrder, QuerySelect, Set,
    sea_query::{Alias, Expr, JoinType, OnConflict, PostgresQueryBuilder, Query},
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};
use uuid::Uuid;

pub async fn get_initial_game_data(
    ctx: &Arc<GatewayContext>,
    level_id: u32,
    persona_id: i32,
) -> Result<InitialGameDataResponse, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();
    // The game client might request an invalid Level ID for some reason...
    // Mimicking the original server's behavior by returning just the player info
    let skip_ugc = level_id != LEVEL_ID_HASH as u32;

    // Fetch User
    let user = users::Entity::find_by_id(persona_id)
        .one(db)
        .await
        .map_err(map_err)?
        .ok_or_else(|| map_err("User not found"))?;

    // Define queries
    let reach_this_query = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .filter(ugc::Column::Type.eq(UgcType::ReachThis));

    let time_trial_query = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .filter(ugc::Column::Type.eq(UgcType::TimeTrial));

    let random_ugc_query = ugc::Entity::find()
        .filter(ugc::Column::Published.eq(true))
        .filter(ugc::Column::AuthorId.ne(persona_id))
        .limit(300);

    let bookmarks_query = ugc_bookmarks::Entity::find()
        .filter(ugc_bookmarks::Column::UserId.eq(persona_id))
        .find_also_related(ugc::Entity);

    let challenge_bm_query = challenge_bookmarks::Entity::find()
        .filter(challenge_bookmarks::Column::UserId.eq(persona_id));

    let (reach_this, time_trials, random_ugc, bookmarks_data, challenge_bookmarks, inventory) =
        if skip_ugc {
            let (inv, c_bm) =
                tokio::try_join!(super::inventory::get_inventory(ctx, persona_id), async {
                    challenge_bm_query.all(db).await.map_err(map_err)
                })?;
            (vec![], vec![], vec![], vec![], c_bm, inv)
        } else {
            tokio::try_join!(
                async { reach_this_query.all(db).await.map_err(map_err) },
                async { time_trial_query.all(db).await.map_err(map_err) },
                async { random_ugc_query.all(db).await.map_err(map_err) },
                async { bookmarks_query.all(db).await.map_err(map_err) },
                async { challenge_bm_query.all(db).await.map_err(map_err) },
                super::inventory::get_inventory(ctx, persona_id),
            )?
        };

    let valid_bookmark_ugcs: Vec<&ugc::Model> = bookmarks_data
        .iter()
        .filter_map(|(_bm, ugc_opt)| ugc_opt.as_ref())
        .collect();

    // Bulk load authors & flags
    // Collect references to all UGC models we are about to process
    let mut all_ugc_refs = Vec::new();
    all_ugc_refs.extend(reach_this.iter());
    all_ugc_refs.extend(time_trials.iter());
    all_ugc_refs.extend(random_ugc.iter());
    all_ugc_refs.extend(valid_bookmark_ugcs);

    let batch_loader = BatchUgcLoader::load(db, persona_id, &all_ugc_refs).await?;

    // 6. Construct Response Objects
    let user_reach_this: Vec<UgcWrapper> = reach_this
        .into_iter()
        .map(|entry| {
            // We know the author is the current user
            UgcWrapper {
                meta: entry.into_meta(&user.name, &UgcFlags::default()),
                stats: None,
                user_stats: None,
                user_rank: None,
            }
        })
        .collect();

    let user_time_trials: Vec<UgcWrapper> = time_trials
        .into_iter()
        .map(|entry| UgcWrapper {
            meta: entry.into_meta(&user.name, &UgcFlags::default()),
            stats: None,
            user_stats: None,
            user_rank: None,
        })
        .collect();

    // Promoted UGC (just Random for now)
    let mut promoted_ugc = Vec::with_capacity(random_ugc.len());
    let mut seen_ids = HashSet::new();

    for entry in random_ugc {
        if seen_ids.insert(entry.id) {
            let author = batch_loader.get_author(entry.author_id);
            let flags = batch_loader.get_flag(&entry.id);
            promoted_ugc.push(PromotedUgcWrapper {
                meta: entry.into_meta(author, &flags),
                reason: 3,
            });
        }
    }

    // Bookmarks
    let ugc_bookmarks_list: Vec<UgcBookmarkEntry> = bookmarks_data
        .into_iter()
        .filter_map(|(bm, ugc_opt)| {
            let entry = ugc_opt?;

            let author = batch_loader.get_author(entry.author_id);
            let flags = batch_loader.get_flag(&entry.id);

            Some(UgcBookmarkEntry {
                ugc_type: entry.r#type.to_string(),
                bookmark_time: bm.bookmark_time.to_string(),
                meta: entry.into_meta(author, &flags),
            })
        })
        .collect();

    let challenge_bookmarks_list: Vec<ChallengeBookmarkEntry> = challenge_bookmarks
        .into_iter()
        .map(|b| ChallengeBookmarkEntry {
            challenge_id: b.challenge_id,
            bookmark_time: b.bookmark_time.to_string(),
            challenge_type: b.challenge_type,
        })
        .collect();

    Ok(InitialGameDataResponse {
        player_info: PlayerInfo {
            name: user.name.clone(),
            division: Division {
                name: user.division_name.clone(),
                rank: user.division_rank,
            },
            location: vec![],
        },
        user_stats: user.stats,
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

    Ok(ugc_model.into_meta(&user.name, &UgcFlags::default()))
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
    let ugc_uuid = Uuid::from_str(&ugc_id).map_err(|_| map_err("Invalid UGC UUID"))?;

    entries::Entity::insert(entries::ActiveModel {
        user_id: Set(persona_id),
        ugc_id: Set(Some(ugc_uuid)),
        ugc_author_id: Set(Some(ugc_author_id)),
        challenge_id: Set(None),
        entry_type: Set(EntryType::ReachThis),
        completed_at: Set(now),
        user_stats: Set(serde_json::Value::Null),
        score: Set(0),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([entries::Column::UserId, entries::Column::UgcId])
            .update_column(entries::Column::CompletedAt)
            .to_owned(),
    )
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

// FIXME:
// ErrorObject { code: InternalError, message: \"Query Error: error returned from database: operator does not exist: entry_type = text\", data: None }
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
                    Expr::col((t1.clone(), entries::Column::EntryType))
                        .eq(Expr::val(EntryType::ReachThis).cast_as(Alias::new("entry_type"))),
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

        let batch_loader =
            BatchUgcLoader::load(db, persona_id, &ugc_entries.iter().collect::<Vec<_>>()).await?;

        for entry in ugc_entries {
            let author = batch_loader.get_author(entry.author_id);
            let flags = batch_loader.get_flag(&entry.id);
            meta_map.insert(entry.id, entry.into_meta(author, &flags));
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
