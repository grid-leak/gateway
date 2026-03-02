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
    #[sea_orm(has_many, from = "persona_id", to = "author_id")]
    pub ugcs: HasMany<super::ugc::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "persona_id")]
    pub accounts: HasMany<super::accounts::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "user_id")]
    pub challenge_entries: HasMany<super::challenge_entries::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "user_id")]
    pub ugc_bookmarks: HasMany<super::ugc_bookmarks::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "user_id")]
    pub challenge_bookmarks: HasMany<super::challenge_bookmarks::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "user_id")]
    pub user_kits: HasMany<super::user_kits::Entity>,
    #[sea_orm(has_many, from = "persona_id", to = "user_id")]
    pub user_ugc_flags: HasMany<super::user_ugc_flags::Entity>,
    #[sea_orm(column_type = "JsonBinary", default = "'{}'")]
    pub tag_data: Json,
}

impl ActiveModelBehavior for ActiveModel {}
