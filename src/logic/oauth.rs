use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::logic::GatewayError;

#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordApplication {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuth2MeResponse {
    pub application: DiscordApplication,
    pub user: DiscordUser,
    pub expires: DateTime<Utc>,
}

pub async fn fetch_discord_user(access_token: &str) -> Result<DiscordUser, GatewayError> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://discord.com/api/oauth2/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| GatewayError::internal(format!("Discord API request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(GatewayError::invalid_params(format!(
            "Discord rejected the access token (HTTP {status})"
        )));
    }

    let oauth_res = response
        .json::<OAuth2MeResponse>()
        .await
        .map_err(|e| GatewayError::internal(format!("Failed to parse Discord OAuth2 response: {e}")))?;

    let expected_client_id = std::env::var("DISCORD_CLIENT_ID")
        .map_err(|_| GatewayError::internal("DISCORD_CLIENT_ID must be set".to_string()))?;

    if oauth_res.application.id != expected_client_id {
        return Err(GatewayError::invalid_params(
            "Access token is for a different client ID".to_string(),
        ));
    }

    if oauth_res.expires < Utc::now() {
        return Err(GatewayError::invalid_params(
            "Access token has expired".to_string(),
        ));
    }

    Ok(oauth_res.user)
}
