use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "user_kits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub user_id: i32,
    pub kit_id: Uuid,
    pub kit_type: Uuid,
    #[sea_orm(default = "false")]
    pub opened: bool,
}

impl ActiveModelBehavior for ActiveModel {}
