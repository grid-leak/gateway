use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "user_ugc_flags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub ugc_id: Uuid,
    #[sea_orm(default = "false")]
    pub reported: bool,
    #[sea_orm(default = "false")]
    pub blocked: bool,
}

impl ActiveModelBehavior for ActiveModel {}
