use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};

// Custom error codes that are properly handled by the game client
#[derive(Debug, Clone, Copy)]
pub enum GameErrorCode {
    // -100: Client should schedule a retry later
    ScheduleRetry,
    // -101: Client should retry immediately
    RetryNow,
    // -32501: Client should re-auth, then retry
    ReAuth,
    // -32502: Hard disconnect (auth failed), no retry
    HardDisconnect,
    NotFound,
    // 200
    TooManyUgc,
    // 201
    TooManyPublishedUgc,
    // 202
    TooManyBookmarks,
    // 203
    UgcNotOwned,
    // 300
    UgcContainsProfanity,
}

impl GameErrorCode {
    fn code(self) -> i32 {
        match self {
            GameErrorCode::ScheduleRetry => -100,
            GameErrorCode::RetryNow => -101,
            GameErrorCode::ReAuth => -32501,
            GameErrorCode::HardDisconnect => -32502,
            GameErrorCode::NotFound => -32503,
            GameErrorCode::TooManyUgc => 200,
            GameErrorCode::TooManyPublishedUgc => 201,
            GameErrorCode::TooManyBookmarks => 202,
            GameErrorCode::UgcNotOwned => 203,
            GameErrorCode::UgcContainsProfanity => 300,
        }
    }
}

#[derive(Debug)]
pub struct GatewayError(ErrorObjectOwned);

impl GatewayError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self(ErrorObject::owned(
            jsonrpsee::types::error::INTERNAL_ERROR_CODE,
            msg.into(),
            None::<()>,
        ))
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self(ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            msg.into(),
            None::<()>,
        ))
    }

    pub fn game(code: GameErrorCode, msg: impl Into<String>) -> Self {
        Self(ErrorObject::owned(code.code(), msg.into(), None::<()>))
    }

    pub fn into_rpc_err(self) -> ErrorObjectOwned {
        self.0
    }
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message())
    }
}

impl From<GatewayError> for ErrorObjectOwned {
    fn from(e: GatewayError) -> Self {
        e.0
    }
}

impl From<sea_orm::DbErr> for GatewayError {
    fn from(e: sea_orm::DbErr) -> Self {
        tracing::error!("Database error: {:?}", e);
        GatewayError::internal("Internal database error")
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(e: serde_json::Error) -> Self {
        GatewayError::internal(e.to_string())
    }
}

impl From<uuid::Error> for GatewayError {
    fn from(e: uuid::Error) -> Self {
        GatewayError::invalid_params(e.to_string())
    }
}

impl From<String> for GatewayError {
    fn from(s: String) -> Self {
        GatewayError::internal(s)
    }
}

impl From<&str> for GatewayError {
    fn from(s: &str) -> Self {
        GatewayError::internal(s)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for GatewayError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        GatewayError::internal(e.to_string())
    }
}
