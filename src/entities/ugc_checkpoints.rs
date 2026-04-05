use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "ugc_checkpoints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ugc_id: Uuid,
    #[sea_orm(belongs_to, from = "ugc_id", to = "id")]
    pub ugc: HasOne<super::ugc::Entity>,
    pub data: Vec<u8>,
}

impl ActiveModelBehavior for ActiveModel {}
