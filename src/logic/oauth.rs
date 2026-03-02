use serde::Deserialize;

use crate::logic::GatewayError;

#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
}

pub async fn fetch_discord_user(access_token: &str) -> Result<DiscordUser, GatewayError> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://discord.com/api/users/@me")
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

    response
        .json::<DiscordUser>()
        .await
        .map_err(|e| GatewayError::internal(format!("Failed to parse Discord user response: {e}")))
}
