use crate::{
    context::GatewayContext,
    entities::{
        challenge_entries::{self, ChallengeEntryType},
        ugc_entries::{self, UgcEntryType},
        users,
    },
    methods::map_err,
    models::game_data::{Division, LeaderboardResponse, LeaderboardUser, LeaderboardWrapper},
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
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
) -> Result<
    crate::models::game_data::OverviewReachThisLeaderboardResponse,
    jsonrpsee::types::ErrorObjectOwned,
> {
    let user = users::Entity::find_by_id(persona_id)
        .one(ctx.db())
        .await
        .map_err(map_err)?
        .ok_or_else(|| map_err("User not found"))?;

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

    Ok(
        crate::models::game_data::OverviewReachThisLeaderboardResponse {
            leaderboard: LeaderboardWrapper {
                area: None,
                total_count,
                users: users_list,
            },
            global_leader,
        },
    )
}

/// TODO: Currently returns the requesting user's own entry only because there
/// is no friends system yet. Once a friends/followers system is implemented,
/// this should filter entries to only include the user's friends and build
/// a proper friends leaderboard
pub async fn get_hackable_billboard_friends_leaderboard(
    ctx: &Arc<GatewayContext>,
    persona_id: i32,
    challenge_id: String,
    offset: i64,
    _count: i64,
) -> Result<LeaderboardResponse, jsonrpsee::types::ErrorObjectOwned> {
    let db = ctx.db();

    let global_count = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::HackableBillboard))
        .count(db)
        .await
        .map_err(map_err)? as i64;

    let global_leader_entry = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::HackableBillboard))
        .order_by_desc(challenge_entries::Column::Score)
        .find_also_related(users::Entity)
        .one(db)
        .await
        .map_err(map_err)?;

    let global_leader = if let Some((entry, Some(user))) = global_leader_entry {
        Some(user_to_leaderboard_entry(
            &user,
            1,
            1,
            entry.completed_at.timestamp_millis().to_string(),
        ))
    } else {
        None
    };

    // TODO: Replace with friends-filtered query once friends system is added
    let user_entry = challenge_entries::Entity::find()
        .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
        .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::HackableBillboard))
        .filter(challenge_entries::Column::UserId.eq(persona_id))
        .find_also_related(users::Entity)
        .one(db)
        .await
        .map_err(map_err)?;

    let mut users_list = Vec::new();
    let mut total_count: i64 = 0;

    if let Some((entry, Some(user))) = user_entry {
        let better_count = challenge_entries::Entity::find()
            .filter(challenge_entries::Column::ChallengeId.eq(&challenge_id))
            .filter(challenge_entries::Column::EntryType.eq(ChallengeEntryType::HackableBillboard))
            .filter(challenge_entries::Column::Score.gt(entry.score))
            .count(db)
            .await
            .map_err(map_err)? as i32;

        let global_rank = better_count + 1;

        // TODO: position within friends leaderboard
        let position = (offset as i32) + 1;

        total_count = 1; // Only our own entry for now

        users_list.push(user_to_leaderboard_entry(
            &user,
            position,
            global_rank,
            entry.completed_at.timestamp_millis().to_string(),
        ));
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
