use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub persona_id: i32,
    pub name: String,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub stats: Json,
    #[sea_orm(default = "'Copper'")]
    pub division_name: String,
    #[sea_orm(default = "5")]
    pub division_rank: i32,
    // TODO: Default value for ghost_variation might be wrong and needs to be checked
    #[sea_orm(default = "2278102450")]
    pub ghost_variation: i32,
    pub ghost_timestamp: DateTimeUtc,
    #[sea_orm(has_many, from = "id", to = "persona_id")]
    pub ugcs: HasMany<super::ugc::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
