use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ugc_type")]
pub enum UgcType {
    #[sea_orm(string_value = "reach_this")]
    ReachThis,
    #[sea_orm(string_value = "time_trial")]
    TimeTrial,
}

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "ugc")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: i32,
    #[sea_orm(belongs_to, from = "author_id", to = "persona_id")]
    pub author: HasOne<super::users::Entity>,
    pub name: String,
    pub r#type: UgcType,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub published: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub qw: f64,
}

impl ActiveModelBehavior for ActiveModel {}
