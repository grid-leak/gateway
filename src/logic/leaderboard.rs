use crate::{
    context::GatewayContext,
    logic::GatewayError,
    models::game_data::{
        Division, LeaderboardResponse, LeaderboardUser, LeaderboardWrapper,
        OverviewLeaderboardResponse,
    },
};
use entities::{
    challenge_entries::{self, ChallengeEntryType},
    ugc_entries::{self, UgcEntryType},
    users,
};
use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select, Related,
};
use std::str::FromStr;
use uuid::Uuid;

trait HasScore {
    fn score(&self) -> i64;
    fn score_display(&self) -> String;
}

impl HasScore for challenge_entries::Model {
    fn score(&self) -> i64 {
        self.score
    }
    fn score_display(&self) -> String {
        self.score.to_string()
    }
}

impl HasScore for ugc_entries::Model {
    fn score(&self) -> i64 {
        self.score
    }
    fn score_display(&self) -> String {
        self.score.to_string()
    }
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

/// Gets the global leader from the first page of entries
fn global_leader_from_page<E: HasScore>(
    page_entries: &[(E, Option<users::Model>)],
) -> Option<LeaderboardUser> {
    page_entries.first().and_then(|(entry, user_opt)| {
        user_opt
            .as_ref()
            .map(|user| user_to_leaderboard_entry(user, 1, 1, entry.score_display()))
    })
}

/// Assembles a page of (entry, Option<user>) pairs into a ranked LeaderboardUser list
fn build_users_list<E: HasScore>(
    page_entries: &[(E, Option<users::Model>)],
    offset: i64,
) -> Vec<LeaderboardUser> {
    page_entries
        .iter()
        .enumerate()
        .filter_map(|(i, (entry, user_opt))| {
            user_opt.as_ref().map(|user| {
                let rank = offset as i32 + i as i32 + 1;
                user_to_leaderboard_entry(user, rank, rank, entry.score_display())
            })
        })
        .collect()
}

fn build_paginated_response(
    users_list: Vec<LeaderboardUser>,
    global_leader: Option<LeaderboardUser>,
    global_count: i64,
) -> LeaderboardResponse {
    LeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count: users_list.len() as i64,
            users: users_list,
        },
        global_leader,
        global_count: global_count.to_string(),
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

fn base_ugc_query(ugc_uuid: Uuid, entry_type: UgcEntryType) -> Select<ugc_entries::Entity> {
    ugc_entries::Entity::find()
        .filter(ugc_entries::Column::UgcId.eq(ugc_uuid))
        .filter(ugc_entries::Column::EntryType.eq(entry_type))
}

async fn execute_leaderboard_pagination<E>(
    db: &sea_orm::DatabaseConnection,
    base_query: Select<E>,
    score_col: E::Column,
    score_order: Order,
    offset: i64,
    count: i64,
) -> Result<LeaderboardResponse, GatewayError>
where
    E: EntityTrait + Related<users::Entity>,
    E::Model: HasScore + sea_orm::FromQueryResult + Sized + Send + Sync + 'static,
{
    let offset = offset.max(0);
    let count = count.max(0);

    let global_count = base_query.clone().count(db).await? as i64;

    let page_entries = base_query
        .clone()
        .order_by(score_col, score_order.clone())
        .offset(offset as u64)
        .limit(count as u64)
        .find_also_related(users::Entity)
        .all(db)
        .await?;

    let global_leader = if offset == 0 {
        global_leader_from_page(&page_entries)
    } else {
        base_query
            .clone()
            .order_by(score_col, score_order.clone())
            .find_also_related(users::Entity)
            .limit(1)
            .one(db)
            .await?
            .and_then(|(entry, user_opt)| {
                user_opt
                    .map(|user| user_to_leaderboard_entry(&user, 1, 1, entry.score_display()))
            })
    };

    let users_list = build_users_list(&page_entries, offset);

    Ok(build_paginated_response(
        users_list,
        global_leader,
        global_count,
    ))
}

async fn execute_overview_leaderboard<E>(
    db: &sea_orm::DatabaseConnection,
    base_query: Select<E>,
    score_col: E::Column,
    user_id_col: E::Column,
    score_order: Order,
    persona_id: i32,
    radius: i32,
) -> Result<OverviewLeaderboardResponse, GatewayError>
where
    E: EntityTrait + Related<users::Entity>,
    E::Model: HasScore + sea_orm::FromQueryResult + Sized + Send + Sync + 'static,
{
    let radius = radius.max(0) as i64;

    let user_entry = base_query
        .clone()
        .filter(user_id_col.eq(persona_id))
        .find_also_related(users::Entity)
        .one(db)
        .await?;

    let user_rank: i64 = if let Some((ref entry, _)) = user_entry {
        let better = match score_order {
            Order::Asc => {
                base_query
                    .clone()
                    .filter(score_col.lt(entry.score()))
                    .count(db)
                    .await?
            }
            Order::Desc => {
                base_query
                    .clone()
                    .filter(score_col.gt(entry.score()))
                    .count(db)
                    .await?
            }
            _ => 0,
        };
        better as i64 + 1
    } else {
        1
    };

    let total_count = base_query.clone().count(db).await? as i64;
    let window_start = (user_rank - 1).saturating_sub(radius).max(0);

    let page_entries = base_query
        .clone()
        .order_by(score_col, score_order.clone())
        .offset(window_start as u64)
        .limit((radius * 2 + 1) as u64)
        .find_also_related(users::Entity)
        .all(db)
        .await?;

    let global_leader = if window_start == 0 {
        global_leader_from_page(&page_entries)
    } else {
        base_query
            .clone()
            .order_by(score_col, score_order.clone())
            .find_also_related(users::Entity)
            .limit(1)
            .one(db)
            .await?
            .and_then(|(entry, user_opt)| {
                user_opt
                    .map(|user| user_to_leaderboard_entry(&user, 1, 1, entry.score_display()))
            })
    };

    let users_list: Vec<LeaderboardUser> = page_entries
        .iter()
        .enumerate()
        .filter_map(|(i, (entry, user_opt))| {
            user_opt.as_ref().map(|user| {
                let global_rank = window_start as i32 + i as i32 + 1;
                let position = i as i32 + 1;
                user_to_leaderboard_entry(user, position, global_rank, entry.score_display())
            })
        })
        .collect();

    Ok(OverviewLeaderboardResponse {
        leaderboard: LeaderboardWrapper {
            area: None,
            total_count,
            users: users_list,
        },
        global_leader,
    })
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

    execute_overview_leaderboard(
        ctx.db(),
        base_ugc_query(ugc_uuid, entry_type),
        ugc_entries::Column::Score,
        ugc_entries::Column::UserId,
        score_order,
        persona_id,
        radius,
    )
    .await
}

pub async fn get_overview_challenge_leaderboard(
    ctx: &GatewayContext,
    persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    radius: i32,
) -> Result<OverviewLeaderboardResponse, GatewayError> {
    execute_overview_leaderboard(
        ctx.db(),
        base_challenge_query(&challenge_id, entry_type),
        challenge_entries::Column::Score,
        challenge_entries::Column::UserId,
        score_order,
        persona_id,
        radius,
    )
    .await
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
    execute_leaderboard_pagination(
        ctx.db(),
        base_challenge_query(&challenge_id, entry_type),
        challenge_entries::Column::Score,
        score_order,
        offset,
        count,
    )
    .await
}

pub async fn get_challenge_friends_leaderboard(
    ctx: &GatewayContext,
    _persona_id: i32,
    challenge_id: String,
    entry_type: ChallengeEntryType,
    score_order: Order,
    offset: i64,
    count: i64,
) -> Result<LeaderboardResponse, GatewayError> {
    // TODO: filter by friends when the friends system is implemented
    execute_leaderboard_pagination(
        ctx.db(),
        base_challenge_query(&challenge_id, entry_type),
        challenge_entries::Column::Score,
        score_order,
        offset,
        count,
    )
    .await
}

pub async fn get_ugc_leaderboard(
    ctx: &GatewayContext,
    _persona_id: i32,
    ugc_id: String,
    entry_type: UgcEntryType,
    score_order: Order,
    offset: i64,
    count: i64,
) -> Result<LeaderboardResponse, GatewayError> {
    let ugc_uuid = Uuid::from_str(&ugc_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid UGC UUID: {e}")))?;

    execute_leaderboard_pagination(
        ctx.db(),
        base_ugc_query(ugc_uuid, entry_type),
        ugc_entries::Column::Score,
        score_order,
        offset,
        count,
    )
    .await
}

pub async fn get_ugc_friends_leaderboard(
    ctx: &GatewayContext,
    _persona_id: i32,
    ugc_id: String,
    entry_type: UgcEntryType,
    score_order: Order,
    offset: i64,
    count: i64,
) -> Result<LeaderboardResponse, GatewayError> {
    // TODO: filter by friends when the friends system is implemented
    let ugc_uuid = Uuid::from_str(&ugc_id)
        .map_err(|e| GatewayError::invalid_params(format!("invalid UGC UUID: {e}")))?;

    execute_leaderboard_pagination(
        ctx.db(),
        base_ugc_query(ugc_uuid, entry_type),
        ugc_entries::Column::Score,
        score_order,
        offset,
        count,
    )
    .await
}
