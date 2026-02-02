use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "challenge_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "user_challenge")]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    #[sea_orm(unique_key = "user_challenge")]
    #[sea_orm(unique_key = "challenge_score")]
    pub challenge_id: String,
    #[sea_orm(unique_key = "challenge_score")]
    pub score: i32,
    #[sea_orm(default_value = "1")]
    pub run_id: i32,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub extra_stats: Json,
    pub created_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
