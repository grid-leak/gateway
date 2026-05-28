use crate::{
    context::GatewayContext,
    logic::{
        GameErrorCode, GatewayError,
        game_data::{UgcFlags, load_ugc_flags, ugc_to_meta, ugc_type_to_string},
    },
    models::{
        game_data::{
            Bookmarks, ChallengeBookmarkEntry, Division, InitialGameDataResponse, Inventory,
            LEVEL_ID_HASH, PlayerInfo, PlayerUgcResponse, PromotedUgcWrapper, ReachThisWrapper,
            TimeTrialWrapper, UgcBookmarkEntry, UgcMeta, UgcWrapper, UserRank,
        },
        ugc::{CreateReachThisMeta, CreateTimeTrialMeta},
        user_stats::{ReachThisUserStats, TimeTrialUserStats, UgcEntryUserStats},
    },
};
use chrono::Utc;
use entities::{
    challenge_bookmarks,
    ugc::{self, UgcType},
    ugc_bookmarks,
    ugc_entries::{self, UgcEntryType},
    users,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter,
    QuerySelect, Set, TransactionTrait,
    sea_query::{Alias, Expr, JoinType, PostgresQueryBuilder, Query},
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use uuid::Uuid;

const UGC_LIMIT: u64 = 300;

fn map_ugc_to_wrapper(
    raw_ugc: Vec<(ugc::Model, Option<users::Model>)>,
    flags_map: &HashMap<Uuid, UgcFlags>,
) -> Vec<UgcWrapper> {
    raw_ugc
        .into_iter()
        .map(|(entry, author_opt)| {
            let author_name = author_opt.as_ref().map(|u| u.name.as_str()).unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            UgcWrapper {
                meta: ugc_to_meta(entry, author_name, &flags),
                stats: None,
                user_stats: None,
                user_rank: None,
            }
        })
        .collect()
}

pub async fn get_initial_game_data(
    ctx: &GatewayContext,
    level_id: i64,
    persona_id: i32,
) -> Result<InitialGameDataResponse, GatewayError> {
    let db = ctx.db();
    let skip_ugc = level_id as i32 != LEVEL_ID_HASH;

    let user = ctx.user(persona_id).await?;

    let player_ugc_query = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .find_also_related(users::Entity);

    let random_ugc_query = ugc::Entity::find()
        .filter(ugc::Column::Published.eq(true))
        .filter(ugc::Column::AuthorId.ne(persona_id))
        .limit(UGC_LIMIT)
        .find_also_related(users::Entity);

    let bookmarks_query = ugc_bookmarks::Entity::find()
        .filter(ugc_bookmarks::Column::UserId.eq(persona_id))
        .find_also_related(ugc::Entity);

    let challenge_bm_query = challenge_bookmarks::Entity::find()
        .filter(challenge_bookmarks::Column::UserId.eq(persona_id));

    let (
        reach_this_raw,
        time_trials_raw,
        random_ugc_raw,
        bookmarks_data,
        challenge_bookmarks,
        inventory,
    ) = if skip_ugc {
        let (inv, c_bm): (Inventory, Vec<challenge_bookmarks::Model>) =
            tokio::try_join!(super::inventory::get_inventory(ctx, persona_id), async {
                challenge_bm_query.all(db).await.map_err(GatewayError::from)
            })?;
        (vec![], vec![], vec![], vec![], c_bm, inv)
    } else {
        let (player_ugcs, random_ugc_raw, bookmarks_data, challenge_bookmarks, inventory) = tokio::try_join!(
            async { player_ugc_query.all(db).await.map_err(GatewayError::from) },
            async { random_ugc_query.all(db).await.map_err(GatewayError::from) },
            async { bookmarks_query.all(db).await.map_err(GatewayError::from) },
            async { challenge_bm_query.all(db).await.map_err(GatewayError::from) },
            super::inventory::get_inventory(ctx, persona_id),
        )?;

        let mut reach_this_raw = Vec::new();
        let mut time_trials_raw = Vec::new();

        for item in player_ugcs {
            match item.0.r#type {
                UgcType::ReachThis => reach_this_raw.push(item),
                UgcType::TimeTrial => time_trials_raw.push(item),
            }
        }

        (
            reach_this_raw,
            time_trials_raw,
            random_ugc_raw,
            bookmarks_data,
            challenge_bookmarks,
            inventory,
        )
    };

    let mut all_ugc_ids: Vec<Uuid> = reach_this_raw
        .iter()
        .map(|(ugc, _)| ugc.id)
        .chain(time_trials_raw.iter().map(|(ugc, _)| ugc.id))
        .chain(random_ugc_raw.iter().map(|(ugc, _)| ugc.id))
        .chain(
            bookmarks_data
                .iter()
                .filter_map(|(_, ugc_opt)| ugc_opt.as_ref().map(|u| u.id)),
        )
        .collect();
    all_ugc_ids.sort_unstable();
    all_ugc_ids.dedup();

    let flags_map = load_ugc_flags(db, persona_id, &all_ugc_ids).await?;

    let user_reach_this = map_ugc_to_wrapper(reach_this_raw, &flags_map);
    let user_time_trials = map_ugc_to_wrapper(time_trials_raw, &flags_map);

    let mut promoted_ugc = Vec::with_capacity(random_ugc_raw.len());
    let mut seen_ids = HashSet::new();

    for (entry, author_opt) in random_ugc_raw {
        if seen_ids.insert(entry.id) {
            let author_name = author_opt.as_ref().map(|u| u.name.as_str()).unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            promoted_ugc.push(PromotedUgcWrapper {
                meta: ugc_to_meta(entry, author_name, &flags),
                reason: 3,
            });
        }
    }

    let bookmark_author_ids: Vec<i32> = bookmarks_data
        .iter()
        .filter_map(|(_, ugc_opt)| ugc_opt.as_ref().map(|u| u.author_id))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let bookmark_authors: HashMap<i32, String> = if bookmark_author_ids.is_empty() {
        HashMap::new()
    } else {
        users::Entity::find()
            .filter(entities::users::Column::PersonaId.is_in(bookmark_author_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|u| (u.persona_id, u.name))
            .collect()
    };

    let ugc_bookmarks_list: Vec<UgcBookmarkEntry> = bookmarks_data
        .into_iter()
        .filter_map(|(bm, ugc_opt)| {
            let entry = ugc_opt?;
            if !entry.published && entry.author_id != persona_id {
                return None;
            }
            let author_name = bookmark_authors
                .get(&entry.author_id)
                .map(|s| s.as_str())
                .unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            Some(UgcBookmarkEntry {
                ugc_type: ugc_type_to_string(&entry.r#type),
                bookmark_time: bm.bookmark_time.timestamp_millis().to_string(),
                meta: ugc_to_meta(entry, author_name, &flags),
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

pub async fn get_player_ugc(
    ctx: &GatewayContext,
    persona_id: i32,
) -> Result<PlayerUgcResponse, GatewayError> {
    let db = ctx.db();

    let player_ugcs = ugc::Entity::find()
        .filter(ugc::Column::AuthorId.eq(persona_id))
        .find_also_related(users::Entity)
        .all(db)
        .await
        .map_err(GatewayError::from)?;

    let mut reach_this_raw = Vec::new();
    let mut time_trials_raw = Vec::new();

    for item in player_ugcs {
        match item.0.r#type {
            UgcType::ReachThis => reach_this_raw.push(item),
            UgcType::TimeTrial => time_trials_raw.push(item),
        }
    }

    let mut all_ugc_ids: Vec<Uuid> = reach_this_raw
        .iter()
        .map(|(ugc, _)| ugc.id)
        .chain(time_trials_raw.iter().map(|(ugc, _)| ugc.id))
        .collect();
    all_ugc_ids.sort_unstable();
    all_ugc_ids.dedup();

    let flags_map = load_ugc_flags(db, persona_id, &all_ugc_ids).await?;

    Ok(PlayerUgcResponse {
        player_reach_this: map_ugc_to_wrapper(reach_this_raw, &flags_map),
        player_time_trials: map_ugc_to_wrapper(time_trials_raw, &flags_map),
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

    Ok(ugc_to_meta(ugc_model, &user.name, &UgcFlags::default()))
}

pub async fn create_time_trial(
    ctx: &GatewayContext,
    author_id: i32,
    data: String,
    time_trial: CreateTimeTrialMeta,
) -> Result<UgcMeta, GatewayError> {
    let db = ctx.db();

    let limits = super::player::get_player_ugc_limits(ctx, author_id).await?;

    if limits.ugc_count >= limits.max_ugc {
        return Err(GatewayError::game(
            GameErrorCode::TooManyUgc,
            "UGC creation limit reached",
        ));
    }
    if time_trial.published && limits.published_count >= limits.max_published {
        return Err(GatewayError::game(
            GameErrorCode::TooManyPublishedUgc,
            "UGC publish limit reached",
        ));
    }

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let decoded_data = STANDARD.decode(&data).map_err(|_| {
        GatewayError::invalid_params("Invalid base64 for time trial checkpoints data")
    })?;

    let user = ctx.user(author_id).await?;

    let now = Utc::now();
    let new_id = Uuid::new_v4();
    let transform = time_trial.transform;

    let ugc_model = db
        .transaction::<_, ugc::Model, GatewayError>(|txn| {
            Box::pin(async move {
                let new_ugc = ugc::ActiveModel {
                    id: Set(new_id),
                    author_id: Set(author_id),
                    name: Set(time_trial.name),
                    r#type: Set(UgcType::TimeTrial),
                    created_at: Set(now),
                    updated_at: Set(now),
                    published: Set(time_trial.published),
                    x: Set(transform.x),
                    y: Set(transform.y),
                    z: Set(transform.z),
                    qx: Set(transform.qx.unwrap_or(0.0)),
                    qy: Set(transform.qy.unwrap_or(0.0)),
                    qz: Set(transform.qz.unwrap_or(0.0)),
                    qw: Set(transform.qw.unwrap_or(1.0)),
                };

                let ugc_model: ugc::Model = new_ugc.insert(txn).await?;

                entities::ugc_checkpoints::ActiveModel {
                    ugc_id: Set(new_id),
                    data: Set(decoded_data),
                }
                .insert(txn)
                .await?;

                Ok(ugc_model)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => GatewayError::from(db_err),
            sea_orm::TransactionError::Transaction(gw_err) => gw_err,
        })?;

    Ok(ugc_to_meta(ugc_model, &user.name, &UgcFlags::default()))
}

pub async fn finish_reach_this(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
) -> Result<(), GatewayError> {
    let db = ctx.db();
    let now = chrono::Utc::now();
    let ugc_uuid =
        Uuid::from_str(&ugc_id).map_err(|_| GatewayError::invalid_params("invalid UGC UUID"))?;

    let metadata = UgcEntryUserStats::ReachThis(ReachThisUserStats {
        reached_at: now.timestamp_millis().to_string(),
    });

    ugc_entries::Entity::insert(ugc_entries::ActiveModel {
        user_id: Set(persona_id),
        ugc_id: Set(ugc_uuid),
        entry_type: Set(UgcEntryType::ReachThis),
        completed_at: Set(now),
        user_stats: Set(serde_json::to_value(&metadata).unwrap_or_default()),
        score: Set(now.timestamp_millis()),
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

    Ok(())
}

pub async fn finish_time_trial(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
    finish_time: i64,
    replay_upload_ticket: String,
    extra_stats: serde_json::Value,
    split_times: Vec<i64>,
) -> Result<(), GatewayError> {
    let db = ctx.db();
    let now = chrono::Utc::now();
    let ugc_uuid =
        Uuid::from_str(&ugc_id).map_err(|_| GatewayError::invalid_params("invalid UGC UUID"))?;

    let existing = ugc_entries::Entity::find()
        .filter(ugc_entries::Column::UserId.eq(persona_id))
        .filter(ugc_entries::Column::UgcId.eq(ugc_uuid))
        .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::TimeTrial))
        .one(db)
        .await?;

    let is_new_record = match &existing {
        Some(entry) => finish_time < entry.score,
        None => true,
    };

    let ticket_key = format!("tickets/{}", replay_upload_ticket);

    let result = async {
        if is_new_record {
            let s3_client = crate::S3_CLIENT.get().expect("S3_CLIENT not initialized");
            let bucket = crate::S3_BUCKET.get().expect("S3_BUCKET not initialized");
            let dest_key = format!("{}/{}", persona_id, ugc_uuid);

            s3_client
                .copy_object()
                .bucket(bucket)
                .key(&dest_key)
                .copy_source(format!("{}/{}", bucket, ticket_key))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to copy S3 object: {:?}", e);
                    GatewayError::internal("failed to process replay upload")
                })?;

            let allowed_keys = [
                "total_distance",
                "walk_distance",
                "maxperframe_distance",
                "wallrun_distance",
            ];

            let extra_stats_map: HashMap<String, String> = extra_stats
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(k, _)| allowed_keys.contains(&k.as_str()))
                        .map(|(k, v)| {
                            let val_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                _ => v.to_string(),
                            };
                            (k.clone(), val_str)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let metadata = UgcEntryUserStats::TimeTrial(TimeTrialUserStats {
                finished_at: now.timestamp_millis().to_string(),
                finish_time: finish_time.to_string(),
                split_times: split_times.iter().map(|s| s.to_string()).collect(),
                extra_stats: extra_stats_map,
            });

            ugc_entries::Entity::insert(ugc_entries::ActiveModel {
                user_id: Set(persona_id),
                ugc_id: Set(ugc_uuid),
                entry_type: Set(UgcEntryType::TimeTrial),
                completed_at: Set(now),
                user_stats: Set(serde_json::to_value(&metadata).unwrap_or_default()),
                score: Set(finish_time),
                ..Default::default()
            })
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    ugc_entries::Column::UserId,
                    ugc_entries::Column::UgcId,
                ])
                .update_columns([
                    ugc_entries::Column::CompletedAt,
                    ugc_entries::Column::UserStats,
                    ugc_entries::Column::Score,
                ])
                .to_owned(),
            )
            .exec(db)
            .await?;
        }

        Ok::<_, GatewayError>(())
    }
    .await;

    let _ = crate::S3_CLIENT
        .get()
        .expect("S3_CLIENT not initialized")
        .delete_object()
        .bucket(crate::S3_BUCKET.get().expect("S3_BUCKET not initialized"))
        .key(&ticket_key)
        .send()
        .await;

    result
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

    let mut user_stats_map = HashMap::new();
    let mut user_ranks_map = HashMap::new();
    let mut totals_map = HashMap::new();

    if data_types.iter().any(|s| s == "USER_STATS") {
        let user_entries = ugc_entries::Entity::find()
            .filter(ugc_entries::Column::UserId.eq(persona_id))
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.iter().copied()))
            .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::ReachThis))
            .all(db)
            .await?;

        for entry in user_entries {
            user_stats_map.insert(entry.ugc_id, entry);
        }

        let totals: Vec<UgcCountResult> = ugc_entries::Entity::find()
            .select_only()
            .column(ugc_entries::Column::UgcId)
            .column_as(ugc_entries::Column::Id.count(), "count")
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.iter().copied()))
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

        if !user_stats_map.is_empty() {
            let t1 = Alias::new("t1");
            let t2 = Alias::new("t2");

            let t1_id = (t1.clone(), ugc_entries::Column::UgcId);
            let t2_id = (t2.clone(), ugc_entries::Column::UgcId);
            let t1_score = (t1.clone(), ugc_entries::Column::CompletedAt);
            let t2_score = (t2.clone(), ugc_entries::Column::CompletedAt);

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

    let mut meta_map = HashMap::new();
    if data_types.iter().any(|s| s == "META") {
        let ugc_rows = ugc::Entity::find()
            .filter(ugc::Column::Id.is_in(requested_ids.iter().copied()))
            .find_also_related(users::Entity)
            .all(db)
            .await?;

        let ugc_ids_for_flags: Vec<Uuid> = ugc_rows.iter().map(|(u, _)| u.id).collect();
        let flags_map = load_ugc_flags(db, persona_id, &ugc_ids_for_flags).await?;

        for (entry, author_opt) in ugc_rows {
            let author_name = author_opt.as_ref().map(|u| u.name.as_str()).unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            meta_map.insert(entry.id, ugc_to_meta(entry, author_name, &flags));
        }
    }

    for ugc_id in ugc_ids {
        let uid = Uuid::from_str(&ugc_id)
            .map_err(|e| GatewayError::invalid_params(format!("invalid UGC UUID: {e}")))?;

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

pub async fn get_time_trial_data(
    ctx: &GatewayContext,
    ugc_ids: Vec<String>,
    data_types: Vec<String>,
    persona_id: i32,
) -> Result<Vec<TimeTrialWrapper>, GatewayError> {
    if ugc_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db = ctx.db();
    let mut responses = Vec::with_capacity(ugc_ids.len());
    let requested_ids: Vec<Uuid> = ugc_ids
        .iter()
        .filter_map(|u| Uuid::from_str(u).ok())
        .collect();

    let mut user_stats_map = HashMap::new();
    let mut user_ranks_map = HashMap::new();
    let mut totals_map = HashMap::new();

    if data_types.iter().any(|s| s == "USER_STATS") {
        let user_entries = ugc_entries::Entity::find()
            .filter(ugc_entries::Column::UserId.eq(persona_id))
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.iter().copied()))
            .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::TimeTrial))
            .all(db)
            .await?;

        for entry in user_entries {
            user_stats_map.insert(entry.ugc_id, entry);
        }

        let totals: Vec<UgcCountResult> = ugc_entries::Entity::find()
            .select_only()
            .column(ugc_entries::Column::UgcId)
            .column_as(ugc_entries::Column::Id.count(), "count")
            .filter(ugc_entries::Column::UgcId.is_in(requested_ids.iter().copied()))
            .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::TimeTrial))
            .group_by(ugc_entries::Column::UgcId)
            .into_model::<UgcCountResult>()
            .all(db)
            .await?;

        for t in totals {
            if let Some(uid) = t.ugc_id {
                totals_map.insert(uid, t.count);
            }
        }

        if !user_stats_map.is_empty() {
            let t1 = Alias::new("t1");
            let t2 = Alias::new("t2");

            let t1_id = (t1.clone(), ugc_entries::Column::UgcId);
            let t2_id = (t2.clone(), ugc_entries::Column::UgcId);
            let t1_score = (t1.clone(), ugc_entries::Column::Score);
            let t2_score = (t2.clone(), ugc_entries::Column::Score);

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
                        .eq(Expr::val(UgcEntryType::TimeTrial)
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

    let mut meta_map = HashMap::new();
    if data_types.iter().any(|s| s == "META") {
        let ugc_rows = ugc::Entity::find()
            .filter(ugc::Column::Id.is_in(requested_ids.iter().copied()))
            .find_also_related(users::Entity)
            .all(db)
            .await?;

        let ugc_ids_for_flags: Vec<Uuid> = ugc_rows.iter().map(|(u, _)| u.id).collect();
        let flags_map = load_ugc_flags(db, persona_id, &ugc_ids_for_flags).await?;

        for (entry, author_opt) in ugc_rows {
            let author_name = author_opt.as_ref().map(|u| u.name.as_str()).unwrap_or("");
            let flags = flags_map.get(&entry.id).cloned().unwrap_or_default();
            meta_map.insert(entry.id, ugc_to_meta(entry, author_name, &flags));
        }
    }

    for ugc_id in ugc_ids {
        let uid = Uuid::from_str(&ugc_id)
            .map_err(|e| GatewayError::invalid_params(format!("invalid UGC UUID: {e}")))?;

        let meta = meta_map.remove(&uid);

        let mut user_stats = None;
        let mut user_rank = None;

        if let Some(entry) = user_stats_map.get(&uid) {
            let parsed_stats_res: Result<TimeTrialUserStats, _> =
                serde_json::from_value(entry.user_stats.clone());

            if let Ok(mut stats) = parsed_stats_res {
                stats.finished_at = entry.completed_at.timestamp_millis().to_string();
                user_stats = Some(stats);
            }

            let better_count = *user_ranks_map.get(&uid).unwrap_or(&0);
            let total_entries = *totals_map.get(&uid).unwrap_or(&0);

            user_rank = Some(UserRank {
                rank: (better_count + 1) as i32,
                score: entry.score.to_string(),
                total: total_entries,
            });
        }

        responses.push(TimeTrialWrapper {
            meta,
            stats: None,
            user_stats,
            user_rank,
        });
    }

    Ok(responses)
}

pub async fn set_ugc_published_flag(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
    published: bool,
) -> Result<bool, GatewayError> {
    let db = ctx.db();
    let ugc_uuid =
        Uuid::from_str(&ugc_id).map_err(|_| GatewayError::invalid_params("invalid UGC UUID"))?;

    let ugc_opt = ugc::Entity::find_by_id(ugc_uuid)
        .one(db)
        .await
        .map_err(GatewayError::from)?;

    let Some(ugc_model) = ugc_opt else {
        return Err(GatewayError::game(GameErrorCode::NotFound, "UGC not found"));
    };

    if ugc_model.author_id != persona_id {
        return Err(GatewayError::game(
            GameErrorCode::UgcNotOwned,
            "You do not own this UGC",
        ));
    }

    if published && !ugc_model.published {
        let limits = super::player::get_player_ugc_limits(ctx, persona_id).await?;
        if limits.published_count >= limits.max_published {
            return Err(GatewayError::game(
                GameErrorCode::TooManyPublishedUgc,
                "UGC publish limit reached",
            ));
        }
    }

    if ugc_model.published != published {
        let now = Utc::now();
        let mut active_model: ugc::ActiveModel = ugc_model.into();
        active_model.published = Set(published);
        active_model.updated_at = Set(now);
        active_model.update(db).await.map_err(GatewayError::from)?;
    }

    Ok(published)
}

pub async fn delete_ugc(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_id: String,
) -> Result<(), GatewayError> {
    let db = ctx.db();
    let ugc_uuid =
        Uuid::from_str(&ugc_id).map_err(|_| GatewayError::invalid_params("invalid UGC UUID"))?;

    let ugc_opt = ugc::Entity::find_by_id(ugc_uuid)
        .one(db)
        .await
        .map_err(GatewayError::from)?;

    let Some(ugc_model) = ugc_opt else {
        return Err(GatewayError::game(GameErrorCode::NotFound, "UGC not found"));
    };

    if ugc_model.author_id != persona_id {
        return Err(GatewayError::game(
            GameErrorCode::UgcNotOwned,
            "You do not own this UGC",
        ));
    }

    ugc::Entity::delete_many()
        .filter(ugc::Column::Id.eq(ugc_uuid))
        .exec(db)
        .await?;

    Ok(())
}
