use crate::{
    context::GatewayContext,
    entities::entries::{self, EntryType},
    models::{
        game_data::{HackableBillboardLeader, RunnersRouteData, UserRank},
        user_stats::{EntryUserStats, HackableBillboardUserStats, RunnersRouteUserStats},
    },
};
use sea_orm::{
    ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, Set,
    sea_query::{Alias, Expr, JoinType, OnConflict, PostgresQueryBuilder, Query},
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, FromQueryResult)]
struct CountResult {
    challenge_id: Option<String>,
    count: i64,
}

pub async fn get_runners_route_data(
    ctx: &Arc<GatewayContext>,
    challenge_ids: Vec<String>,
    _data_types: Vec<String>,
    persona_id: i32,
) -> Result<Vec<RunnersRouteData>, String> {
    if challenge_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db = ctx.db();

    // Fetch requested entries for the user
    let user_entries = entries::Entity::find()
        .filter(entries::Column::UserId.eq(persona_id))
        .filter(entries::Column::ChallengeId.is_in(&challenge_ids))
        .filter(entries::Column::EntryType.eq(EntryType::RunnersRoute))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let entries_map: HashMap<String, entries::Model> = user_entries
        .into_iter()
        .filter_map(|e| e.challenge_id.clone().map(|id| (id, e)))
        .collect();

    // Fetch total entry counts
    let totals: Vec<CountResult> = entries::Entity::find()
        .select_only()
        .column(entries::Column::ChallengeId)
        .column_as(entries::Column::Id.count(), "count")
        .filter(entries::Column::ChallengeId.is_in(&challenge_ids))
        .filter(entries::Column::EntryType.eq(EntryType::RunnersRoute))
        .group_by(entries::Column::ChallengeId)
        .into_model::<CountResult>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let totals_map: HashMap<String, i64> = totals
        .into_iter()
        .filter_map(|t| t.challenge_id.map(|id| (id, t.count)))
        .collect();

    // Fetch and calculate ranks
    let ranks_map =
        if !entries_map.is_empty() {
            let t1 = Alias::new("t1"); // Persona
            let t2 = Alias::new("t2"); // Others

            let t1_id = (t1.clone(), entries::Column::ChallengeId);
            let t2_id = (t2.clone(), entries::Column::ChallengeId);
            let t1_score = (t1.clone(), entries::Column::Score);
            let t2_score = (t2.clone(), entries::Column::Score);

            // Join t2 where Challenge IDs match AND t2.score < t1.score.
            // This counts how many people have a LOWER score than the user.
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
                .and_where(Expr::col(t1_id.clone()).is_in(entries_map.keys().cloned()))
                .and_where(
                    Expr::col((t1.clone(), entries::Column::EntryType)).eq(EntryType::RunnersRoute),
                )
                .group_by_col(t1_id);

            let (sql, values) = query.build(PostgresQueryBuilder);

            let rank_results = CountResult::find_by_statement(
                sea_orm::Statement::from_sql_and_values(DbBackend::Postgres, &sql, values),
            )
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

            rank_results
                .into_iter()
                .filter_map(|r| r.challenge_id.map(|id| (id, r.count)))
                .collect::<HashMap<String, i64>>()
        } else {
            HashMap::new()
        };

    let mut responses = Vec::with_capacity(challenge_ids.len());

    for challenge_id in challenge_ids {
        let mut user_stats = None;
        let mut user_rank = None;

        if let Some(entry) = entries_map.get(&challenge_id) {
            let stats: RunnersRouteUserStats = serde_json::from_value(entry.user_stats.clone())
                .map_err(|e| format!("Failed to parse user stats for {}: {}", challenge_id, e))?;

            let better_count = *ranks_map.get(&challenge_id).unwrap_or(&0);
            let total_entries = *totals_map.get(&challenge_id).unwrap_or(&0);

            user_rank = Some(UserRank {
                rank: (better_count + 1) as i32,
                score: stats.finish_time.to_string(),
                total: total_entries,
            });

            user_stats = Some(stats);
        }

        responses.push(RunnersRouteData {
            id: challenge_id,
            stats: None,
            user_stats,
            user_rank,
        });
    }

    Ok(responses)
}

// TODO: revisit this after adding friends & followers system
// currently it's global and returns the latest user that has an entry
// which could potentially be more interesting than the original server
pub async fn get_hackable_billboard_friends_leaders(
    ctx: &Arc<GatewayContext>,
    challenge_ids: Vec<String>,
) -> Result<HashMap<String, Option<HackableBillboardLeader>>, String> {
    if challenge_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let db = ctx.db();
    let mut response_map = HashMap::new();

    for challenge_id in challenge_ids {
        let entry_opt = entries::Entity::find()
            .filter(entries::Column::ChallengeId.eq(&challenge_id))
            .filter(entries::Column::EntryType.eq(EntryType::HackableBillboard))
            .order_by_desc(entries::Column::CompletedAt)
            .find_also_related(crate::entities::users::Entity)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if let Some((entry, Some(user))) = entry_opt {
            response_map.insert(
                challenge_id,
                Some(HackableBillboardLeader {
                    position: 1,
                    score: entry.completed_at.timestamp_millis().to_string(),
                    persona_id: user.persona_id.to_string(),
                    name: user.name,
                }),
            );
        } else {
            response_map.insert(challenge_id, None);
        }
    }

    Ok(response_map)
}

pub async fn finish_hackable_billboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    challenge_id: String,
    main_stat: i32,
    _extra_stats: serde_json::Value,
) -> Result<String, String> {
    let db = ctx.db();
    let now = chrono::Utc::now();

    let metadata = EntryUserStats::HackableBillboard(HackableBillboardUserStats {
        finished_at: main_stat.to_string(),
    });

    entries::Entity::insert(entries::ActiveModel {
        user_id: Set(persona_id),
        challenge_id: Set(Some(challenge_id)),
        ugc_id: Set(None),
        entry_type: Set(EntryType::HackableBillboard),
        completed_at: Set(now),
        user_stats: Set(serde_json::to_value(&metadata).unwrap_or_default()),
        score: Set(main_stat),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([entries::Column::UserId, entries::Column::ChallengeId])
            .update_columns([entries::Column::CompletedAt, entries::Column::UserStats])
            .to_owned(),
    )
    .exec(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok("success".to_string())
}
