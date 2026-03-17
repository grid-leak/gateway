use crate::{
    context::GatewayContext,
    entities::{
        challenge_entries::{self, ChallengeEntryType},
        ugc_entries::{self, UgcEntryType},
        users,
    },
    logic::GatewayError,
    models::game_data::{
        Division, LeaderboardResponse, LeaderboardUser, LeaderboardWrapper,
        OverviewLeaderboardResponse,
    },
};
use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use std::str::FromStr;
use uuid::Uuid;

struct RankedEntry {
    user_id: i32,
    score_display: String,
    user: users::Model,
}

/// Maps a user model + score data into the API response
fn user_to_leaderboard_entry(
    user: &users::Model,
    position: i32,
    global_rank: i32,
    score: String,
) -> LeaderboardUser {
    LeaderboardUser {
        position,
        global_rank,
        score,
        percentile: None,
        persona_id: user.persona_id.to_string(),
        name: user.name.clone(),
        division: Division {
            name: user.division_name.clone(),
            rank: user.division_rank,
        },
    }
}

/// Given a storted list, return the bounds for a window of `radius` entries around the center
fn centered_window(total: i32, center: i32, radius: i32) -> (usize, usize) {
    let start = (center - radius).max(0) as usize;
    let end = (center + radius).min((total - 1).max(0)) as usize;
    (start, end)
}

/// Finds the requested persona in the leaderboard, slices a window around them
/// and assembles the Overview API Response
fn build_overview_response(
    entries: Vec<RankedEntry>,
    persona_id: i32,
    radius: i32,
) -> OverviewLeaderboardResponse {
    let total = entries.len() as i32;

    let center = entries
        .iter()
        .position(|e| e.user_id == persona_id)
        .map(|i| i as i32)
        .unwrap_or(0);

    let (start, end) = centered_window(total, center, radius);

    let users_list: Vec<LeaderboardUser> = if total > 0 {
        entries[start..=end]
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let global_rank = (start + i) as i32 + 1;
                let position = i as i32 + 1;
                user_to_leaderboard_entry(
                    &entry.user,
                    position,
                    global_rank,
                    entry.score_display.clone(),
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    let global_leader = entries
        .first()
        .map(|entry| user_to_leaderboard_entry(&entry.user, 1, 1, entry.score_display.clone()));

    OverviewLeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count: total as i64,
            users: users_list,
        },
        global_leader,
    }
}

fn base_challenge_query(
    challenge_id: &str,
    entry_type: ChallengeEntryType,
) -> Select<challenge_entries::Entity> {
    challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(entry_type))
}

pub async fn get_overview_ugc_leaderboard(
    ctx: &GatewayContext,
    persona_id: i32,
    ugc_uuid: String,
    entry_type: UgcEntryType,
    score_order: Order,
    radius: i32,
) -> Result<OverviewLeaderboardResponse, GatewayError> {
    let ugc_uuid = Uuid::from_str(&ugc_uuid)
        .map_err(|e| GatewayError::invalid_params(format!("invalid UGC UUID: {e}")))?;

    let all_entries = ugc_entries::Entity::find()
        .filter(ugc_entries::Column::UgcId.eq(ugc_uuid))
        .filter(ugc_entries::Column::EntryType.eq(entry_type))
        .order_by(ugc_entries::Column::Score, score_order)
        .find_also_related(users::Entity)
        .all(ctx.db())
        .await?;

    let ranked: Vec<RankedEntry> = all_entries
        .into_iter()
        .filter_map(|(entry, user_opt)| {
            user_opt.map(|user| RankedEntry {
                user_id: user.persona_id,
                score_display: entry.score.to_string(),
                user,
            })
        })
        .collect();

    Ok(build_overview_response(ranked, persona_id, radius.max(0)))
}

pub async fn get_overview_challenge_leaderboard(
    ctx: &GatewayContext,
    persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    radius: i32,
) -> Result<OverviewLeaderboardResponse, GatewayError> {
    let all_entries = base_challenge_query(&challenge_id, entry_type)
        .order_by(challenge_entries::Column::Score, score_order)
        .find_also_related(users::Entity)
        .all(ctx.db())
        .await?;

    let ranked: Vec<RankedEntry> = all_entries
        .into_iter()
        .filter_map(|(entry, user_opt)| {
            user_opt.map(|user| RankedEntry {
                user_id: user.persona_id,
                score_display: entry.score.to_string(),
                user,
            })
        })
        .collect();

    Ok(build_overview_response(ranked, persona_id, radius.max(0)))
}

pub async fn get_challenge_leaderboard(
    ctx: &GatewayContext,
    _persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    offset: i64,
    count: i64,
) -> Result<LeaderboardResponse, GatewayError> {
    let db = ctx.db();

    let global_count = base_challenge_query(&challenge_id, entry_type)
        .count(db)
        .await? as i64;

    let page_entries = base_challenge_query(&challenge_id, entry_type)
        .order_by(challenge_entries::Column::Score, score_order.clone())
        .offset(offset as u64)
        .limit(count as u64)
        .find_also_related(users::Entity)
        .all(db)
        .await?;

    let global_leader = if offset == 0 {
        page_entries.first().and_then(|(entry, user_opt)| {
            user_opt
                .as_ref()
                .map(|user| user_to_leaderboard_entry(user, 1, 1, entry.score.to_string()))
        })
    } else {
        base_challenge_query(&challenge_id, entry_type)
            .order_by(challenge_entries::Column::Score, score_order)
            .find_also_related(users::Entity)
            .one(db)
            .await?
            .and_then(|(entry, user_opt)| {
                user_opt.map(|user| user_to_leaderboard_entry(&user, 1, 1, entry.score.to_string()))
            })
    };

    let users_list: Vec<LeaderboardUser> = page_entries
        .into_iter()
        .enumerate()
        .filter_map(|(i, (entry, user_opt))| {
            user_opt.map(|user| {
                let rank = (offset as i32) + (i as i32) + 1;
                user_to_leaderboard_entry(&user, rank, rank, entry.score.to_string())
            })
        })
        .collect();

    let returned_count = users_list.len() as i64;

    Ok(LeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count: returned_count,
            users: users_list,
        },
        global_leader,
        global_count: global_count.to_string(),
    })
}

pub async fn get_challenge_friends_leaderboard(
    ctx: &GatewayContext,
    persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    offset: i64,
) -> Result<LeaderboardResponse, GatewayError> {
    let db = ctx.db();

    let global_count = base_challenge_query(&challenge_id, entry_type)
        .count(db)
        .await? as i64;

    let global_leader = base_challenge_query(&challenge_id, entry_type)
        .order_by(challenge_entries::Column::Score, score_order.clone())
        .find_also_related(users::Entity)
        .one(db)
        .await?
        .and_then(|(entry, user_opt)| {
            user_opt.map(|user| user_to_leaderboard_entry(&user, 1, 1, entry.score.to_string()))
        });

    let user_entry = base_challenge_query(&challenge_id, entry_type)
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .find_also_related(users::Entity)
        .one(db)
        .await?;

    let mut users_list = Vec::new();
    let mut total_count: i64 = 0;

    if let Some((entry, Some(user))) = user_entry {
        let better_count = match score_order {
            Order::Asc => {
                base_challenge_query(&challenge_id, entry_type)
                    .filter(challenge_entries::Column::Score.lt(entry.score))
                    .count(db)
                    .await? as i32
            }
            Order::Desc => {
                base_challenge_query(&challenge_id, entry_type)
                    .filter(challenge_entries::Column::Score.gt(entry.score))
                    .count(db)
                    .await? as i32
            }
            _ => 0,
        };

        let global_rank = better_count + 1;
        let position = (offset as i32) + 1;

        let percentile = if global_count > 0 {
            Some(((global_count - global_rank as i64) as f64 / global_count as f64 * 100.0).floor())
        } else {
            None
        };

        total_count = 1;

        let mut lb_user =
            user_to_leaderboard_entry(&user, position, global_rank, entry.score.to_string());
        lb_user.percentile = percentile;

        users_list.push(lb_user);
    }

    Ok(LeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count,
            users: users_list,
        },
        global_leader,
        global_count: global_count.to_string(),
    })
}
