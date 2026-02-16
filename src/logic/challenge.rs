use crate::{
    context::GatewayContext,
    entities::{
        challenge_entries::{self, ChallengeEntryType},
        users,
    },
    models::{
        game_data::{Division, HackableBillboardLeader, RunnersRouteData, UserRank},
        user_stats::{ChallengeEntryUserStats, HackableBillboardUserStats, RunnersRouteUserStats},
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, ExprTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Set, sea_query::{Alias, Expr, JoinType, OnConflict, PostgresQueryBuilder, Query}
};
use std::collections::HashMap;
use std::sync::Arc;

struct RunnersRouteThreshold {
    id: &'static str,
    threshold_1star: i32,
    threshold_2star: i32,
    threshold_3star: i32,
}

static RUNNERS_ROUTE_THRESHOLDS: &[RunnersRouteThreshold] = &[
    RunnersRouteThreshold {
        id: "ch_rrt_tv2_time",
        threshold_1star: 4600,
        threshold_2star: 3800,
        threshold_3star: 3500,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_rz3_time",
        threshold_1star: 2650,
        threshold_2star: 2300,
        threshold_3star: 2100,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc4_time",
        threshold_1star: 8000,
        threshold_2star: 4750,
        threshold_3star: 3600,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_trtt1_time",
        threshold_1star: 2000,
        threshold_2star: 1500,
        threshold_3star: 1000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt5_time",
        threshold_1star: 6400,
        threshold_2star: 5800,
        threshold_3star: 4600,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc2_time",
        threshold_1star: 5100,
        threshold_2star: 4350,
        threshold_3star: 2350,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dte3_time",
        threshold_1star: 6000,
        threshold_2star: 5000,
        threshold_3star: 4000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc6_time",
        threshold_1star: 3300,
        threshold_2star: 2900,
        threshold_3star: 2400,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc5_time",
        threshold_1star: 22000,
        threshold_2star: 19000,
        threshold_3star: 12900,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_mcity1_time",
        threshold_1star: 6000,
        threshold_2star: 5000,
        threshold_3star: 4000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_tv3_time",
        threshold_1star: 9500,
        threshold_2star: 7400,
        threshold_3star: 6450,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt1_time",
        threshold_1star: 6000,
        threshold_2star: 5200,
        threshold_3star: 4500,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt2_time",
        threshold_1star: 4600,
        threshold_2star: 3800,
        threshold_3star: 3250,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_bm1_time",
        threshold_1star: 5600,
        threshold_2star: 4800,
        threshold_3star: 4000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc1_time",
        threshold_1star: 4400,
        threshold_2star: 4100,
        threshold_3star: 1400,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt6_time",
        threshold_1star: 4400,
        threshold_2star: 3800,
        threshold_3star: 3400,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_truxrr1_time",
        threshold_1star: 1500,
        threshold_2star: 1000,
        threshold_3star: 500,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_tv04_time",
        threshold_1star: 11800,
        threshold_2star: 8600,
        threshold_3star: 5700,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_trtt2_time",
        threshold_1star: 3000,
        threshold_2star: 2000,
        threshold_3star: 1000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt3_time",
        threshold_1star: 4200,
        threshold_2star: 3400,
        threshold_3star: 1900,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_anc3_time",
        threshold_1star: 5800,
        threshold_2star: 5100,
        threshold_3star: 3750,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_demo1_time",
        threshold_1star: 6000,
        threshold_2star: 5000,
        threshold_3star: 4000,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_tv05_time",
        threshold_1star: 14200,
        threshold_2star: 10600,
        threshold_3star: 9600,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_rz4_time",
        threshold_1star: 2800,
        threshold_2star: 2500,
        threshold_3star: 2150,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_tv1_time",
        threshold_1star: 6800,
        threshold_2star: 5900,
        threshold_3star: 5500,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_rz1_time",
        threshold_1star: 3350,
        threshold_2star: 3000,
        threshold_3star: 2850,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_rz2_time",
        threshold_1star: 5700,
        threshold_2star: 4500,
        threshold_3star: 4050,
    },
    RunnersRouteThreshold {
        id: "ch_rrt_dt4_time",
        threshold_1star: 6200,
        threshold_2star: 5200,
        threshold_3star: 4400,
    },
];

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
    let user_entries = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .filter(challenge_entries::Column::ChallengeId.is_in(&challenge_ids))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::RunnersRoute))
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
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::RunnersRoute))
        .group_by(challenge_entries::Column::ChallengeId)
        .into_model::<CountResult>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let totals_map: HashMap<String, i64> = totals
        .into_iter()
        .filter_map(|t| t.challenge_id.map(|id| (id, t.count)))
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
            .and_where(Expr::col((t1.clone(), challenge_entries::Column::UserId)).eq(persona_id))
            .and_where(Expr::col(t1_id.clone()).is_in(entries_map.keys().cloned()))
            .and_where(
                Expr::col((t1.clone(), challenge_entries::Column::EntryType))
                    .eq(ChallengeEntryType::RunnersRoute),
            )
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
        let entry_opt = challenge_entries::Entity::find()
            .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
            .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::HackableBillboard))
            .order_by_desc(challenge_entries::Column::CompletedAt)
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

    let metadata = ChallengeEntryUserStats::HackableBillboard(HackableBillboardUserStats {
        finished_at: main_stat.to_string(),
    });

    challenge_entries::Entity::insert(challenge_entries::ActiveModel {
        user_id: Set(persona_id),
        challenge_id: Set(challenge_id),
        entry_type: Set(ChallengeEntryType::HackableBillboard),
        completed_at: Set(now),
        user_stats: Set(serde_json::to_value(&metadata).unwrap_or_default()),
        score: Set(main_stat),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([
            challenge_entries::Column::UserId,
            challenge_entries::Column::ChallengeId,
        ])
        .update_columns([
            challenge_entries::Column::CompletedAt,
            challenge_entries::Column::UserStats,
        ])
        .to_owned(),
    )
    .exec(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok("success".to_string())
}

fn calculate_stars(challenge_id: &str, score: i32) -> u32 {
    let threshold = match RUNNERS_ROUTE_THRESHOLDS
        .iter()
        .find(|t| t.id == challenge_id)
    {
        Some(t) => t,
        None => return 0,
    };

    if score <= threshold.threshold_3star {
        3
    } else if score <= threshold.threshold_2star {
        2
    } else if score <= threshold.threshold_1star {
        1
    } else {
        0
    }
}

// Calculate division from total stars across all RunnersRoute challenges
// Max possible 84 stars (28 challenges x 3 stars)
fn calculate_division(total_stars: u32) -> Division {
    // Each division has 5 ranks, each rank covers ~3.36 stars
    // Use ceiling division so even 1 star gets you into Copper 4
    const DIVISIONS: [&str; 5] = ["Copper", "Bronze", "Silver", "Gold", "Red"];
    const STARS_PER_RANK: f64 = 84.0 / 25.0; // ~3.36

    if total_stars == 0 {
        return Division {
            name: "Copper".to_string(),
            rank: 5,
        };
    }

    let tier = std::cmp::min(
        ((total_stars as f64) / STARS_PER_RANK).ceil() as u32 - 1,
        24,
    );
    let division_index = (tier / 5) as usize;
    let rank = 5 - (tier % 5) as i32;

    Division {
        name: DIVISIONS[division_index].to_string(),
        rank,
    }
}

pub async fn finish_runners_route(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    challenge_id: String,
    main_stat: i32,
    extra_stats: serde_json::Value,
    run_id: i32,
) -> Result<Division, String> {
    if !RUNNERS_ROUTE_THRESHOLDS
        .iter()
        .any(|t| t.id == challenge_id)
    {
        return Err(format!(
            "Invalid RunnersRoute challenge ID: {}",
            challenge_id
        ));
    }

    let db = ctx.db();
    let now = chrono::Utc::now();

    let extra_stats_map: HashMap<String, String> = extra_stats
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let metadata = ChallengeEntryUserStats::RunnersRoute(RunnersRouteUserStats {
        finished_at: now.timestamp_millis().to_string(),
        finish_time: main_stat.to_string(),
        extra_stats: extra_stats_map,
        run_id: run_id.to_string(),
    });

    let existing = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::RunnersRoute))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    let should_upsert = match &existing {
        // Only update if new score is better
        Some(entry) => main_stat < entry.score,
        None => true,
    };

    if should_upsert {
        challenge_entries::Entity::insert(challenge_entries::ActiveModel {
            user_id: Set(persona_id),
            challenge_id: Set(challenge_id),
            entry_type: Set(ChallengeEntryType::RunnersRoute),
            completed_at: Set(now),
            user_stats: Set(serde_json::to_value(&metadata).unwrap_or_default()),
            score: Set(main_stat),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([
                challenge_entries::Column::UserId,
                challenge_entries::Column::ChallengeId,
            ])
            .update_columns([
                challenge_entries::Column::CompletedAt,
                challenge_entries::Column::UserStats,
                challenge_entries::Column::Score,
            ])
            .to_owned(),
        )
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    }

    let all_entries = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::RunnersRoute))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let total_stars: u32 = all_entries
        .iter()
        .map(|e| calculate_stars(&e.challenge_id, e.score))
        .sum();

    let division = calculate_division(total_stars);

    let mut user: users::ActiveModel = users::Entity::find_by_id(persona_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?
        .into();

    user.division_name = Set(division.name.clone());
    user.division_rank = Set(division.rank);
    user.update(db).await.map_err(|e| e.to_string())?;

    Ok(division)
}
