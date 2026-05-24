use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::logic::GatewayError;
use entities::users;

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

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn register_session(&self, session_id: String, persona_id: i32) {
        let mut sessions = self.sessions.lock().expect("failed to lock sessions");
        let session = Session {
            persona_id,
            created_at: Instant::now(),
        };
        sessions.insert(session_id, session);
    }

    pub fn get_persona_id(&self, session_id: &str) -> Option<i32> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            if session.created_at.elapsed() < SESSION_LIFETIME {
                Some(session.persona_id)
            } else {
                sessions.remove(session_id);
                None
            }
        } else {
            None
        }
    }

    pub async fn user(&self, persona_id: i32) -> Result<users::Model, GatewayError> {
        users::Entity::find_by_id(persona_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| GatewayError::internal("user not found"))
    }

    pub fn purge_expired_sessions(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, session| session.created_at.elapsed() < SESSION_LIFETIME);
    }
}
