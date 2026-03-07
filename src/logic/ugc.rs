use crate::{
    context::GatewayContext,
    entities::{
        challenge_bookmarks,
        ugc::{self, UgcType},
        ugc_bookmarks,
        ugc_entries::{self, UgcEntryType},
    },
    logic::{
        GameErrorCode, GatewayError,
        game_data::{BatchUgcLoader, UgcFlags},
    },
    models::{
        game_data::{
            Bookmarks, ChallengeBookmarkEntry, Division, InitialGameDataResponse, Inventory,
            LEVEL_ID_HASH, PlayerInfo, PromotedUgcWrapper, ReachThisWrapper, UgcBookmarkEntry,
            UgcMeta, UgcWrapper, UserRank,
        },
        ugc::CreateReachThisMeta,
        user_stats::ReachThisUserStats,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, ModelTrait,
    QueryFilter, QuerySelect, Set,
    sea_query::{Alias, Expr, JoinType, PostgresQueryBuilder, Query},
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use uuid::Uuid;

pub async fn get_initial_game_data(
    ctx: &GatewayContext,
    level_id: u32,
    persona_id: i32,
) -> Result<InitialGameDataResponse, GatewayError> {
    let db = ctx.db();
    // The game client might request an invalid Level ID for some reason...
    // Mimicking the original server's behavior by returning just the player info
    let skip_ugc = level_id != LEVEL_ID_HASH as u32;

    // Fetch User
    let user = ctx.user(persona_id).await?;

    // Define queries
    let reach_this_query: sea_orm::Select<ugc::Entity> = user
        .find_related(ugc::Entity)
        .filter(ugc::Column::Type.eq(UgcType::ReachThis));

    let time_trial_query: sea_orm::Select<ugc::Entity> = user
        .find_related(ugc::Entity)
        .filter(ugc::Column::Type.eq(UgcType::TimeTrial));

    let random_ugc_query = ugc::Entity::find()
        .filter(ugc::Column::Published.eq(true))
        .filter(ugc::Column::AuthorId.ne(persona_id))
        .limit(300);

    let bookmarks_query = user
        .find_related(ugc_bookmarks::Entity)
        .find_also_related(ugc::Entity);

    let challenge_bm_query: sea_orm::Select<challenge_bookmarks::Entity> =
        user.find_related(challenge_bookmarks::Entity);

    let (reach_this, time_trials, random_ugc, bookmarks_data, challenge_bookmarks, inventory) =
        if skip_ugc {
            let (inv, c_bm): (Inventory, Vec<challenge_bookmarks::Model>) =
                tokio::try_join!(super::inventory::get_inventory(ctx, persona_id), async {
                    challenge_bm_query.all(db).await.map_err(GatewayError::from)
                })?;
            (vec![], vec![], vec![], vec![], c_bm, inv)
        } else {
            tokio::try_join!(
                async { reach_this_query.all(db).await.map_err(GatewayError::from) },
                async { time_trial_query.all(db).await.map_err(GatewayError::from) },
                async { random_ugc_query.all(db).await.map_err(GatewayError::from) },
                async { bookmarks_query.all(db).await.map_err(GatewayError::from) },
                async { challenge_bm_query.all(db).await.map_err(GatewayError::from) },
                super::inventory::get_inventory(ctx, persona_id),
            )?
        };

    let valid_bookmark_ugcs: Vec<&ugc::Model> = bookmarks_data
        .iter()
        .filter_map(|(_bm, ugc_opt): &(ugc_bookmarks::Model, Option<ugc::Model>)| ugc_opt.as_ref())
        .collect();

    // Bulk load authors & flags
    // Collect references to all UGC models we are about to process
    let mut all_ugc_refs: Vec<&ugc::Model> = Vec::new();
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
                bookmark_time: bm.bookmark_time.timestamp_millis().to_string(),
                meta: entry.into_meta(author, &flags),
            })
        })
        .collect();

    let challenge_bookmarks_list: Vec<ChallengeBookmarkEntry> = challenge_bookmarks
        .into_iter()
        .map(|b| ChallengeBookmarkEntry {
            challenge_id: b.challenge_id,
            bookmark_time: b.bookmark_time.timestamp_millis().to_string(),
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
) -> Result<UgcMeta, GatewayError> {
    let db = ctx.db();

    let limits = super::player::get_player_ugc_limits(ctx, author_id).await?;

    if limits.ugc_count >= limits.max_ugc {
        return Err(GatewayError::game(
            GameErrorCode::TooManyUgc,
            "UGC creation limit reached",
        ));
    }
    if reach_this.published && limits.published_count >= limits.max_published {
        return Err(GatewayError::game(
            GameErrorCode::TooManyPublishedUgc,
            "UGC publish limit reached",
        ));
    }

    let user = ctx.user(author_id).await?;

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

    let ugc_model: ugc::Model = new_ugc.insert(db).await?;

    Ok(ugc_model.into_meta(&user.name, &UgcFlags::default()))
}

pub async fn finish_reach_this(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
) -> Result<ReachThisWrapper, GatewayError> {
    let db = ctx.db();
    let now = chrono::Utc::now();
    let ugc_uuid =
        Uuid::from_str(&ugc_id).map_err(|_| GatewayError::invalid_params("invalid UGC UUID"))?;

    ugc_entries::Entity::insert(ugc_entries::ActiveModel {
        user_id: Set(persona_id),
        ugc_id: Set(ugc_uuid),
        entry_type: Set(UgcEntryType::ReachThis),
        completed_at: Set(now),
        user_stats: Set(serde_json::Value::Null),
        score: Set(0),
        ..Default::default()
    })
    .on_conflict(
        sea_orm::sea_query::OnConflict::columns([
            ugc_entries::Column::UserId,
            ugc_entries::Column::UgcId,
        ])
        .update_column(ugc_entries::Column::CompletedAt)
        .to_owned(),
    )
    .exec(db)
    .await?;

    // UGC Meta needs to be returned in the response or
    // the game will erase the Beat LE from the map
    let ugc_model = ugc::Entity::find_by_id(ugc_uuid)
        .one(db)
        .await?
        .ok_or_else(|| GatewayError::internal("UGC not found"))?;

    let batch_loader = BatchUgcLoader::load(db, persona_id, &[&ugc_model]).await?;
    let author_name = batch_loader.get_author(ugc_model.author_id);
    let flags = batch_loader.get_flag(&ugc_model.id);

    Ok(ReachThisWrapper {
        meta: Some(ugc_model.into_meta(author_name, &flags)),
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
struct UgcCountResult {
    ugc_id: Option<Uuid>,
    count: i64,
}

pub async fn get_reach_this_data(
    ctx: &GatewayContext,
    ugc_ids: Vec<String>,
    data_types: Vec<String>,
    persona_id: i32,
) -> Result<Vec<ReachThisWrapper>, GatewayError> {
    if ugc_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db = ctx.db();
    let mut responses = Vec::with_capacity(ugc_ids.len());
    let requested_ids: Vec<Uuid> = ugc_ids
        .iter()
        .filter_map(|u| Uuid::from_str(u).ok())
        .collect();

    // 1. Fetch User Stats & Ranks if requested
    let mut user_stats_map = HashMap::new();
    let mut user_ranks_map = HashMap::new();
    let mut totals_map = HashMap::new();

    if data_types.iter().any(|s| s == "USER_STATS") {
        // Fetch user entries
        let user_entries = ugc_entries::Entity::find()
            .filter(ugc_entries::Column::UserId.eq(persona_id))
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.clone()))
            .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::ReachThis))
            .all(db)
            .await?;

        for entry in user_entries {
            user_stats_map.insert(entry.ugc_id, entry);
        }

        // Fetch totals
        let totals: Vec<UgcCountResult> = ugc_entries::Entity::find()
            .select_only()
            .column(ugc_entries::Column::UgcId)
            .column_as(ugc_entries::Column::Id.count(), "count")
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.clone()))
            .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::ReachThis))
            .group_by(ugc_entries::Column::UgcId)
            .into_model::<UgcCountResult>()
            .all(db)
            .await?;

        for t in totals {
            if let Some(uid) = t.ugc_id {
                totals_map.insert(uid, t.count);
            }
        }

        // Fetch Ranks (Completed At Ascending - earlier is better)
        if !user_stats_map.is_empty() {
            let t1 = Alias::new("t1");
            let t2 = Alias::new("t2");

            let t1_id = (t1.clone(), ugc_entries::Column::UgcId);
            let t2_id = (t2.clone(), ugc_entries::Column::UgcId);
            let t1_score = (t1.clone(), ugc_entries::Column::CompletedAt);
            let t2_score = (t2.clone(), ugc_entries::Column::CompletedAt);

            // Count how many people have completed_at < my completed_at
            let join_condition = Expr::col(t1_id.clone())
                .equals(t2_id.clone())
                .and(Expr::col(t2_score).lt(Expr::col(t1_score)));

            let mut query = Query::select();
            query
                .column(t1_id.clone())
                .expr_as(
                    Expr::col((t2.clone(), ugc_entries::Column::Id)).count(),
                    "count",
                )
                .from_as(ugc_entries::Entity, t1.clone())
                .join_as(
                    JoinType::LeftJoin,
                    ugc_entries::Entity,
                    t2.clone(),
                    join_condition,
                )
                .and_where(Expr::col((t1.clone(), ugc_entries::Column::UserId)).eq(persona_id))
                .and_where(Expr::col(t1_id.clone()).is_in(requested_ids.clone()))
                .and_where(
                    Expr::col((t1.clone(), ugc_entries::Column::EntryType))
                        .eq(Expr::val(UgcEntryType::ReachThis)
                            .cast_as(Alias::new("ugc_entry_type"))),
                )
                .group_by_col(t1_id);

            let (sql, values) = query.build(PostgresQueryBuilder);

            let rank_results = UgcCountResult::find_by_statement(
                sea_orm::Statement::from_sql_and_values(DbBackend::Postgres, &sql, values),
            )
            .all(db)
            .await?;

            for r in rank_results {
                if let Some(uid) = r.ugc_id {
                    user_ranks_map.insert(uid, r.count);
                }
            }
        }
    }

    // 2. Fetch Meta if requested
    let mut meta_map = HashMap::new();
    if data_types.iter().any(|s| s == "META") {
        let ugc_entries = ugc::Entity::find()
            .filter(ugc::Column::Id.is_in(requested_ids.clone()))
            .all(db)
            .await?;

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
