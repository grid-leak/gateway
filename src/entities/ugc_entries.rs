use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ugc_entry_type")]
pub enum UgcEntryType {
    #[sea_orm(string_value = "ReachThis")]
    ReachThis,
    #[sea_orm(string_value = "TimeTrial")]
    TimeTrial,
}

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "ugc_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "user_ugc_unique")]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    #[sea_orm(unique_key = "user_ugc_unique")]
    pub ugc_id: Uuid,
    #[sea_orm(belongs_to, from = "ugc_id", to = "id")]
    pub ugc: HasOne<super::ugc::Entity>,
    pub entry_type: UgcEntryType,
    #[sea_orm(indexed)]
    pub completed_at: DateTimeUtc,
    pub score: i64,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub user_stats: Json,
}

impl ActiveModelBehavior for ActiveModel {}
