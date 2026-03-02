use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub persona_id: i32,
    pub name: String,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub stats: Json,
    #[sea_orm(default = "'Copper'")]
    pub division_name: String,
    #[sea_orm(default = "5")]
    pub division_rank: i32,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub ghost_data: Json,
    #[sea_orm(has_many, from = "id", to = "persona_id")]
    pub ugcs: HasMany<super::ugc::Entity>,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub tag_data: Json,
}

impl ActiveModelBehavior for ActiveModel {}
