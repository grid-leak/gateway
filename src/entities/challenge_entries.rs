use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    enum_name = "challenge_entry_type"
)]
pub enum ChallengeEntryType {
    #[sea_orm(string_value = "HackableBillboard")]
    HackableBillboard,
    #[sea_orm(string_value = "RunnersRoute")]
    RunnersRoute,
}

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "challenge_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "user_challenge_unique")]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    #[sea_orm(unique_key = "user_challenge_unique")]
    pub challenge_id: String,
    pub entry_type: ChallengeEntryType,
    #[sea_orm(indexed)]
    pub completed_at: DateTimeUtc,
    pub score: i32,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub user_stats: Json,
}

impl ActiveModelBehavior for ActiveModel {}
