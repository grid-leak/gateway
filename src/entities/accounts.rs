use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub persona_id: i32,
    #[sea_orm(belongs_to, from = "persona_id", to = "persona_id")]
    pub user: HasOne<super::users::Entity>,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_username: String,
}

impl ActiveModelBehavior for ActiveModel {}
