use diesel::Selectable;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::Queryable;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::SmallInt;
use serde::Serialize;

// Custom Types
pub type PersonaId = i64;

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = SmallInt)]
pub enum UgcType {
    ReachThis,
    TimeTrial,
}

impl ToSql<SmallInt, Pg> for UgcType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            UgcType::ReachThis => <i16 as ToSql<SmallInt, Pg>>::to_sql(&0, out),
            UgcType::TimeTrial => <i16 as ToSql<SmallInt, Pg>>::to_sql(&1, out),
        }
    }
}

impl FromSql<SmallInt, Pg> for UgcType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(UgcType::ReachThis),
            1 => Ok(UgcType::TimeTrial),
            n => Err(format!("Unknown UGC type: {}", n).into()),
        }
    }
}

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = SmallInt)]
pub enum Division {
    Copper,
    Bronze,
    Silver,
    Gold,
    Red,
}

impl ToSql<SmallInt, Pg> for Division {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            Division::Copper => <i16 as ToSql<SmallInt, Pg>>::to_sql(&0, out),
            Division::Bronze => <i16 as ToSql<SmallInt, Pg>>::to_sql(&1, out),
            Division::Silver => <i16 as ToSql<SmallInt, Pg>>::to_sql(&2, out),
            Division::Gold => <i16 as ToSql<SmallInt, Pg>>::to_sql(&3, out),
            Division::Red => <i16 as ToSql<SmallInt, Pg>>::to_sql(&4, out),
        }
    }
}

impl FromSql<SmallInt, Pg> for Division {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(Division::Copper),
            1 => Ok(Division::Bronze),
            2 => Ok(Division::Silver),
            3 => Ok(Division::Gold),
            4 => Ok(Division::Red),
            n => Err(format!("Unknown Division: {}", n).into()),
        }
    }
}

// Database Models

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: PersonaId,
    pub name: String,
    pub division_name: Division,
    pub division_rank: i16,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_stats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct UserStats {
    pub user_id: PersonaId,
    pub stats: serde_json::Value,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::ugcs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct Ugc {
    pub id: uuid::Uuid,
    pub user_id: PersonaId,
    pub name: String,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub published: bool,
    pub type_id: UgcType,
    pub transform: serde_json::Value,
    pub map_position: Option<serde_json::Value>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::ugc_bookmarks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Ugc, foreign_key = ugc_id))]
#[diesel(primary_key(user_id, ugc_id))]
pub struct UgcBookmark {
    pub user_id: PersonaId,
    pub ugc_id: uuid::Uuid,
    pub bookmark_time: std::time::SystemTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::challenge_bookmarks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(primary_key(user_id, challenge_id))]
pub struct ChallengeBookmark {
    pub user_id: PersonaId,
    pub challenge_id: uuid::Uuid,
    pub bookmark_time: std::time::SystemTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::kit_unlocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(primary_key(user_id, kit_id))]
pub struct KitUnlock {
    pub user_id: PersonaId,
    pub kit_id: uuid::Uuid,
    pub opened: bool,
}

// API Models

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub persona_id: String,
    pub name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub tag: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagData {
    pub frame: Tag,
    pub bg: Tag,
    pub detail: Tag,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPlayerTagResponse {
    pub persona_id: String,
    pub tag_data: TagData,
}
