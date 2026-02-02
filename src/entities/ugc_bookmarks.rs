use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "ugc_bookmarks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub ugc_id: Uuid,
    pub bookmark_time: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
