use crate::{
    context::GatewayContext,
    entities::challenge_entries,
    models::challenge::{RunnersRouteDataResponse, UserRank, UserStats},
};
use sea_orm::{
    ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect,
    sea_query::{Alias, Expr, JoinType, OnConflict, PostgresQueryBuilder, Query},
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, FromQueryResult)]
struct CountResult {
    challenge_id: String,
    count: i64,
}

pub async fn get_runners_route_data(
    ctx: &Arc<GatewayContext>,
    challenge_ids: Vec<String>,
    _data_types: Vec<String>,
    persona_id: i32,
) -> Result<Vec<RunnersRouteDataResponse>, String> {
    if challenge_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db = ctx.db();

    // Fetch requested challenge entries
    let user_entries = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .filter(challenge_entries::Column::ChallengeId.is_in(&challenge_ids))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let entries_map: HashMap<String, challenge_entries::Model> = user_entries
        .into_iter()
        .map(|e| (e.challenge_id.clone(), e))
        .collect();

    // Fetch total entry counts
    let totals: Vec<CountResult> = challenge_entries::Entity::find()
        .select_only()
        .column(challenge_entries::Column::ChallengeId)
        .column_as(challenge_entries::Column::Id.count(), "count")
        .filter(challenge_entries::Column::ChallengeId.is_in(&challenge_ids))
        .group_by(challenge_entries::Column::ChallengeId)
        .into_model::<CountResult>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let totals_map: HashMap<String, i64> = totals
        .into_iter()
        .map(|t| (t.challenge_id, t.count))
        .collect();

    // Fetch and calculate ranks
    let ranks_map = if !entries_map.is_empty() {
        let t1 = Alias::new("t1"); // Persona
        let t2 = Alias::new("t2"); // Others

        let t1_id = (t1.clone(), challenge_entries::Column::ChallengeId);
        let t2_id = (t2.clone(), challenge_entries::Column::ChallengeId);
        let t1_score = (t1.clone(), challenge_entries::Column::Score);
        let t2_score = (t2.clone(), challenge_entries::Column::Score);

        // Join t2 where Challenge IDs match AND t2.score < t1.score.
        // This counts how many people have a LOWER score than the user.
        let join_condition = Expr::col(t1_id.clone())
            .equals(t2_id.clone())
            .and(Expr::col(t2_score).lt(Expr::col(t1_score)));

        let mut query = Query::select();
        query
            .column(t1_id.clone())
            .expr_as(
                Expr::col((t2.clone(), challenge_entries::Column::Id)).count(),
                "count",
            )
            .from_as(challenge_entries::Entity, t1.clone())
            .join_as(
                JoinType::LeftJoin,
                challenge_entries::Entity,
                t2.clone(),
                join_condition,
            )
            // Only look for our user and the challenges we actually have entries for
            .and_where(Expr::col((t1.clone(), challenge_entries::Column::UserId)).eq(persona_id))
            .and_where(Expr::col(t1_id.clone()).is_in(entries_map.keys().cloned()))
            .group_by_col(t1_id);

        let (sql, values) = query.build(PostgresQueryBuilder);

        let rank_results = CountResult::find_by_statement(sea_orm::Statement::from_sql_and_values(
            DbBackend::Postgres,
            &sql,
            values,
        ))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

        rank_results
            .into_iter()
            .map(|r| (r.challenge_id, r.count))
            .collect::<HashMap<String, i64>>()
    } else {
        HashMap::new()
    };

    let mut responses = Vec::with_capacity(challenge_ids.len());

    for challenge_id in challenge_ids {
        let mut user_stats = None;
        let mut user_rank = None;

        if let Some(entry) = entries_map.get(&challenge_id) {
            let extra_stats = serde_json::from_value(entry.extra_stats.clone()).unwrap_or_default();

            user_stats = Some(UserStats {
                finished_at: entry.created_at.timestamp_millis().to_string(),
                finish_time: entry.score.to_string(),
                extra_stats,
                run_id: entry.run_id.to_string(),
            });

            let better_score_count = *ranks_map.get(&challenge_id).unwrap_or(&0);
            let total_entries = *totals_map.get(&challenge_id).unwrap_or(&0);

            user_rank = Some(UserRank {
                rank: (better_score_count + 1) as i32,
                score: entry.score.to_string(),
                total: total_entries,
            });
        }

        responses.push(RunnersRouteDataResponse {
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
) -> Result<HashMap<String, Option<crate::models::challenge::HackableBillboardLeader>>, String> {
    if challenge_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let db = ctx.db();
    let mut response_map = HashMap::new();

    for challenge_id in challenge_ids {
        let entry_opt = challenge_entries::Entity::find()
            .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
            .order_by_desc(challenge_entries::Column::CreatedAt)
            .find_also_related(crate::entities::users::Entity)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if let Some((entry, Some(user))) = entry_opt {
            response_map.insert(
                challenge_id,
                Some(crate::models::challenge::HackableBillboardLeader {
                    position: 1,
                    score: entry.created_at.timestamp_millis().to_string(),
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
    score: i32,
    extra_stats: serde_json::Value,
) -> Result<String, String> {
    let db = ctx.db();
    let now = chrono::Utc::now();

    challenge_entries::Entity::insert(challenge_entries::ActiveModel {
        user_id: sea_orm::Set(persona_id),
        challenge_id: sea_orm::Set(challenge_id),
        score: sea_orm::Set(score),
        extra_stats: sea_orm::Set(extra_stats),
        created_at: sea_orm::Set(now),
        run_id: sea_orm::Set(1),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([
            challenge_entries::Column::UserId,
            challenge_entries::Column::ChallengeId,
        ])
        .update_columns([
            challenge_entries::Column::Score,
            challenge_entries::Column::ExtraStats,
            challenge_entries::Column::CreatedAt,
        ])
        .to_owned(),
    )
    .exec(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok("success".to_string())
}
