use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "entry_type")]
pub enum EntryType {
    #[sea_orm(string_value = "HackableBillboard")]
    HackableBillboard,
    #[sea_orm(string_value = "RunnersRoute")]
    RunnersRoute,
    #[sea_orm(string_value = "ReachThis")]
    ReachThis,
    #[sea_orm(string_value = "TimeTrial")]
    TimeTrial,
}

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "user_challenge_unique", unique_key = "user_ugc_unique")]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    #[sea_orm(unique_key = "user_ugc_unique")]
    pub ugc_id: Option<Uuid>,
    // TODO: get rid of ugc_author_id
    pub ugc_author_id: Option<i32>,
    #[sea_orm(unique_key = "user_challenge_unique")]
    pub challenge_id: Option<String>,
    pub entry_type: EntryType,
    #[sea_orm(indexed)]
    pub completed_at: DateTimeUtc,
    // Main stat for leaderboard calculations
    // reachedAt for ReachThis
    // finishedAt for HackableBillboard
    // finishTime for RunnersRoute and TimeTrial
    pub score: i32,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub user_stats: Json,
}

impl ActiveModelBehavior for ActiveModel {}
