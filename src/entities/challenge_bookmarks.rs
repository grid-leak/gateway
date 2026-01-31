use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "challenge_bookmarks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub challenge_id: String,
    pub challenge_type: String,
    pub bookmark_time: i64,
}

impl ActiveModelBehavior for ActiveModel {}
