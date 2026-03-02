use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "user_ugc_flags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    #[sea_orm(primary_key, auto_increment = false)]
    pub ugc_id: Uuid,
    #[sea_orm(belongs_to, from = "ugc_id", to = "id")]
    pub ugc: HasOne<super::ugc::Entity>,
    #[sea_orm(default = "false")]
    pub reported: bool,
    #[sea_orm(default = "false")]
    pub blocked: bool,
}

impl ActiveModelBehavior for ActiveModel {}
