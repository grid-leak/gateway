use crate::{
    context::GatewayContext,
    entities::{
        challenge_entries::{self, ChallengeEntryType},
        ugc_entries::{self, UgcEntryType},
        users,
    },
    methods::map_err,
    models::game_data::{
        Division, LeaderboardResponse, LeaderboardUser, LeaderboardWrapper,
        OverviewLeaderboardResponse,
    },
};
use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

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

pub async fn get_overview_reach_this_leaderboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    ugc_uuid: String,
    radius: Option<i32>,
) -> Result<OverviewLeaderboardResponse, jsonrpsee::types::ErrorObjectOwned> {
    let user = ctx.user(persona_id).await?;

    let db = ctx.db();
    let ugc_uuid = Uuid::from_str(&ugc_uuid).map_err(map_err)?;
    let radius = std::cmp::max(radius.unwrap_or(3), 0);

    let all_entries = ugc_entries::Entity::find()
        .filter(ugc_entries::Column::UgcId.eq(ugc_uuid))
        .filter(ugc_entries::Column::EntryType.eq(UgcEntryType::ReachThis))
        .order_by_asc(ugc_entries::Column::CompletedAt)
        .all(db)
        .await
        .map_err(map_err)?;

    let total_count = all_entries.len() as i64;

    let center_rank = all_entries
        .iter()
        .position(|e| e.user_id == persona_id)
        .map(|idx| idx as i32)
        .unwrap_or(0);

    let start_rank = std::cmp::max(center_rank - radius, 0);
    let end_rank = std::cmp::min(center_rank + radius, (total_count - 1).max(0) as i32);

    let slice = if total_count > 0 {
        &all_entries[start_rank as usize..=end_rank as usize]
    } else {
        &[]
    };

    let mut users_list = Vec::with_capacity(slice.len());

    for (i, entry) in slice.iter().enumerate() {
        let rank = start_rank + i as i32 + 1;
        users_list.push(user_to_leaderboard_entry(
            &user,
            rank,
            rank,
            entry.completed_at.timestamp_millis().to_string(),
        ));
    }

    let global_leader = if total_count > 0 {
        let entry = &all_entries[0];
        Some(user_to_leaderboard_entry(
            &user,
            1,
            1,
            entry.completed_at.timestamp_millis().to_string(),
        ))
    } else {
        None
    };

    Ok(OverviewLeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count,
            users: users_list,
        },
        global_leader,
    })
}

pub async fn get_overview_challenge_leaderboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    radius: i32,
) -> Result<OverviewLeaderboardResponse, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();
    let radius = std::cmp::max(radius, 0);

    let all_entries = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(entry_type))
        .order_by(challenge_entries::Column::Score, score_order)
        .find_also_related(users::Entity)
        .all(db)
        .await
        .map_err(map_err)?;

    let total_count = all_entries.len() as i64;

    let center_index = all_entries
        .iter()
        .position(|(_entry, user)| {
            user.as_ref()
                .map(|u| u.persona_id == persona_id)
                .unwrap_or(false)
        })
        .map(|idx| idx as i32)
        .unwrap_or(0);

    let start_index = std::cmp::max(center_index - radius, 0) as usize;
    let end_index = std::cmp::min(center_index + radius, (total_count - 1).max(0) as i32) as usize;

    let mut users_list = Vec::new();

    if total_count > 0 {
        for (i, (entry, user_opt)) in all_entries[start_index..=end_index].iter().enumerate() {
            if let Some(user) = user_opt {
                let global_rank = (start_index + i) as i32 + 1;
                users_list.push(user_to_leaderboard_entry(
                    user,
                    global_rank,
                    global_rank,
                    entry.score.to_string(),
                ));
            }
        }
    }

    let global_leader = all_entries.first().and_then(|(entry, user_opt)| {
        user_opt
            .as_ref()
            .map(|user| user_to_leaderboard_entry(user, 1, 1, entry.score.to_string()))
    });

    Ok(OverviewLeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count,
            users: users_list,
        },
        global_leader,
    })
}

/// TODO: For friends_only, returns the requesting user's own entry only because there
/// is no friends system yet. Once a friends/followers system is implemented,
/// this should filter entries to only include the user's friends.
pub async fn get_challenge_leaderboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    offset: i64,
    count: i64,
    friends_only: bool,
) -> Result<LeaderboardResponse, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    // 1. Global Count
    let global_count = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(entry_type.clone()))
        .count(db)
        .await
        .map_err(map_err)? as i64;

    // 2. Global Leader
    let global_leader_entry = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(entry_type.clone()))
        .order_by(challenge_entries::Column::Score, score_order.clone())
        .find_also_related(users::Entity)
        .one(db)
        .await
        .map_err(map_err)?;

    let global_leader = if let Some((entry, Some(user))) = global_leader_entry {
        Some(user_to_leaderboard_entry(
            &user,
            1,
            1,
            entry.score.to_string(),
        ))
    } else {
        None
    };

    let mut users_list = Vec::new();
    let mut total_count: i64 = 0;

    if friends_only {
        // TODO: Replace with friends-filtered query once friends system is added
        // Existing behavior: Return ONLY the current user
        let user_entry = challenge_entries::Entity::find()
            .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
            .filter(challenge_entries::Column::EntryType.eq(entry_type.clone()))
            .filter(challenge_entries::Column::UserId.eq(persona_id))
            .find_also_related(users::Entity)
            .one(db)
            .await
            .map_err(map_err)?;

        if let Some((entry, Some(user))) = user_entry {
            // Count how many have a better score than the user to determine rank
            let better_count = match score_order {
                Order::Asc => {
                    // Lower is better → count entries with score < user's score
                    challenge_entries::Entity::find()
                        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
                        .filter(challenge_entries::Column::EntryType.eq(entry_type))
                        .filter(challenge_entries::Column::Score.lt(entry.score))
                        .count(db)
                        .await
                        .map_err(map_err)? as i32
                }
                Order::Desc => {
                    // Higher is better → count entries with score > user's score
                    challenge_entries::Entity::find()
                        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
                        .filter(challenge_entries::Column::EntryType.eq(entry_type))
                        .filter(challenge_entries::Column::Score.gt(entry.score))
                        .count(db)
                        .await
                        .map_err(map_err)? as i32
                }
                _ => 0,
            };

            let global_rank = better_count + 1;
            // logic::leaderboard::get_challenge_friends_leaderboard assumes
            // the returned list position is relative to the requested offset if it was a list
            // but for "friends only" (single user), position is often just 1 or based on offset?
            // In the original code: `let position = (offset as i32) + 1;`
            let position = (offset as i32) + 1;

            // Calculate percentile: percentage of players the user is better than
            let percentile = if global_count > 0 {
                Some(
                    ((global_count as i64 - global_rank as i64) as f64 / global_count as f64
                        * 100.0)
                        .floor(),
                )
            } else {
                None
            };

            total_count = 1; // Only our own entry for now

            let mut lb_user =
                user_to_leaderboard_entry(&user, position, global_rank, entry.score.to_string());
            lb_user.percentile = percentile;

            users_list.push(lb_user);
        }
    } else {
        // Return top N entries (paginated by offset/count)
        let entries = challenge_entries::Entity::find()
            .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
            .filter(challenge_entries::Column::EntryType.eq(entry_type.clone()))
            .order_by(challenge_entries::Column::Score, score_order)
            .offset(offset as u64)
            .limit(count as u64)
            .find_also_related(users::Entity)
            .all(db)
            .await
            .map_err(map_err)?;

        // total_count for this logic often implies the *returned* count or the *total available*?
        // LeaderboardWrapper usually wants the total count of the list being returned or the scope.
        // In `get_overview_challenge_leaderboard`, total_count is the length of `all_entries`.
        // Let's set it to the count of users we are returning.
        total_count = entries.len() as i64;

        for (i, (entry, user_opt)) in entries.iter().enumerate() {
            if let Some(user) = user_opt {
                let rank = (offset as i32) + (i as i32) + 1;
                // For a straight list, position = rank usually
                users_list.push(user_to_leaderboard_entry(
                    user,
                    rank,
                    rank,
                    entry.score.to_string(),
                ));
            }
        }
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
