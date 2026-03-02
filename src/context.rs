use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::entities::users;
use crate::logic::GatewayError;

const SESSION_LIFETIME: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
struct Session {
    persona_id: i32,
    created_at: Instant,
}

#[derive(Clone)]
pub struct GatewayContext {
    db: DatabaseConnection,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl GatewayContext {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // TODO: explore switching to "services", eg. find_user, find_ugc, etc
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn register_session(&self, session_id: String, persona_id: i32) {
        let mut sessions = self.sessions.lock().unwrap();
        let session = Session {
            persona_id,
            created_at: Instant::now(),
        };
        sessions.insert(session_id, session);
    }

    pub fn get_persona_id(&self, session_id: &str) -> Option<i32> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id)
            && session.created_at.elapsed() < SESSION_LIFETIME
        {
            return Some(session.persona_id);
        }
        sessions.remove(session_id);
        None
    }

    pub async fn user(&self, persona_id: i32) -> Result<users::Model, GatewayError> {
        users::Entity::find_by_id(persona_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| GatewayError::internal("user not found"))
    }
}
