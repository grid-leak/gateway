use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "ugc_bookmarks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(
        belongs_to,
        from = "user_id",
        to = "persona_id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    pub user: HasOne<super::users::Entity>,

    #[sea_orm(primary_key, auto_increment = false)]
    pub ugc_id: Uuid,
    #[sea_orm(
        belongs_to,
        from = "ugc_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    pub ugc: HasOne<super::ugc::Entity>,

    pub bookmark_time: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
