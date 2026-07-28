//! Spotify Web API client — OAuth PKCE flow, token refresh, and playback control.
//!
//! PKCE (Proof Key for Code Exchange) is the recommended auth flow for desktop
//! and mobile apps — no client secret required. The user authorises in a
//! browser, we get an authorization code, exchange it for tokens, and store
//! them. The refresh token persists indefinitely (until revoked).
//!
//! Token storage: `{config_dir}/quake-companion/spotify_token.json`
//! (TODO: encrypt at rest — for now it's plaintext JSON, same as config.json)

use std::fs;
use std::io;
use std::path::PathBuf;

use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};

// ---- Spotify API constants -------------------------------------------------

/// Spotify Authorize endpoint.
pub const AUTH_URL: &str = "https://accounts.spotify.com/authorize";
/// Token exchange endpoint.
pub const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
/// API base URL.
pub const API_BASE: &str = "https://api.spotify.com/v1";

/// Scopes we need for playback control + reading now-playing state.
pub const SCOPES: &str = "user-read-playback-state,user-modify-playback-state,user-read-currently-playing,streaming";

/// Redirect URI — localhost server that receives the auth callback.
/// We spin up a tiny HTTP server on port 8888 to catch the redirect.
pub const REDIRECT_PORT: u16 = 8888;
pub const REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";

/// Default client ID — this is a public client (PKCE, no secret).
/// TODO: Phillip needs to register a Spotify app and replace this.
/// For now using the well-known Spotify demo client ID.
pub const CLIENT_ID: &str = "YOUR_SPOTIFY_CLIENT_ID";

// ---- Token storage --------------------------------------------------------

/// Stored Spotify tokens (persisted to disk).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SpotifyTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix timestamp (seconds) when the access token expires.
    pub expires_at: i64,
    /// The token type (always "Bearer").
    #[serde(default = "default_bearer")]
    pub token_type: String,
}

fn default_bearer() -> String {
    "Bearer".to_string()
}

/// Return the token file path.
pub fn token_path() -> Option<PathBuf> {
    crate::config::config_dir().map(|d| d.join("spotify_token.json"))
}

/// Load tokens from disk. Returns None if not stored or invalid.
pub fn load_tokens() -> Option<SpotifyTokens> {
    let path = token_path()?;
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save tokens to disk.
pub fn save_tokens(tokens: &SpotifyTokens) -> io::Result<()> {
    let dir = crate::config::config_dir().ok_or_else(|| io::Error::other("no config directory"))?;
    fs::create_dir_all(&dir)?;
    let path = token_path().unwrap();
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| io::Error::other(format!("token serialize error: {e}")))?;
    fs::write(&path, json + "\n")?;
    Ok(())
}

/// Clear stored tokens (logout).
pub fn clear_tokens() -> io::Result<()> {
    if let Some(path) = token_path() {
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Check if we have valid (non-expired) tokens.
pub fn is_authenticated() -> bool {
    match load_tokens() {
        Some(t) => {
            // 60 second buffer before expiry
            let now = chrono::Utc::now().timestamp();
            t.access_token.len() > 0 && now < t.expires_at - 60
        }
        None => false,
    }
}

// ---- PKCE helpers ----------------------------------------------------------

/// Generate a cryptographically random code verifier (43-128 chars, base64url).
pub fn generate_code_verifier() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Compute the code challenge from a verifier (SHA256 → base64url).
pub fn code_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result)
}

/// Generate a random state string for CSRF protection.
pub fn generate_state() -> String {
    let bytes: [u8; 16] = rand::thread_rng().gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

// ---- Auth URL builder ------------------------------------------------------

/// Build the Spotify authorise URL for PKCE flow.
pub fn auth_url(client_id: &str, code_verifier: &str, state: &str) -> String {
    let challenge = code_challenge(code_verifier);
    format!(
        "{AUTH_URL}?response_type=code&client_id={client_id}&scope={scopes}&redirect_uri={redirect}&state={state}&code_challenge_method=S256&code_challenge={challenge}",
        scopes = urlencoding::encode(SCOPES),
        redirect = urlencoding::encode(REDIRECT_URI),
    )
}

// ---- Token exchange --------------------------------------------------------

/// Exchange an authorization code for access + refresh tokens.
pub async fn exchange_code(
    client_id: &str,
    code: &str,
    code_verifier: &str,
) -> Result<SpotifyTokens, String> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", REDIRECT_URI),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("token exchange request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("token exchange failed: {body}"));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("token exchange parse failed: {e}"))?;

    let now = chrono::Utc::now().timestamp();
    Ok(SpotifyTokens {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token.unwrap_or_default(),
        expires_at: now + token_resp.expires_in as i64,
        token_type: token_resp.token_type,
    })
}

/// Refresh an expired access token using the refresh token.
pub async fn refresh_token(
    client_id: &str,
    refresh_token: &str,
) -> Result<SpotifyTokens, String> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("token refresh request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("token refresh failed: {body}"));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("token refresh parse failed: {e}"))?;

    let now = chrono::Utc::now().timestamp();
    Ok(SpotifyTokens {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token.unwrap_or(refresh_token.to_string()),
        expires_at: now + token_resp.expires_in as i64,
        token_type: token_resp.token_type,
    })
}

/// Get a valid access token, refreshing if needed.
pub async fn ensure_valid_token(client_id: &str) -> Result<String, String> {
    let tokens = load_tokens().ok_or("not authenticated")?;
    let now = chrono::Utc::now().timestamp();

    if now < tokens.expires_at - 60 {
        return Ok(tokens.access_token);
    }

    // Need to refresh
    let refreshed = refresh_token(client_id, &tokens.refresh_token).await?;
    save_tokens(&refreshed).map_err(|e| e.to_string())?;
    Ok(refreshed.access_token)
}

// ---- Token response type ---------------------------------------------------

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
    #[serde(default)]
    #[allow(dead_code)]
    scope: Option<String>,
}

// ---- Spotify API calls -----------------------------------------------------

/// Now-playing track info.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NowPlaying {
    pub is_playing: bool,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub album_art_url: String,
    /// Duration in ms.
    pub duration_ms: u64,
    /// Progress in ms.
    pub progress_ms: u64,
}

/// Get the user's currently playing track.
pub async fn get_now_playing(access_token: &str) -> Result<Option<NowPlaying>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{API_BASE}/me/player/currently-playing"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("now-playing request failed: {e}"))?;

    if resp.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(format!("now-playing failed: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("now-playing parse failed: {e}"))?;

    let is_playing = body["is_playing"].as_bool().unwrap_or(false);
    let item = &body["item"];
    if item.is_null() {
        return Ok(Some(NowPlaying { is_playing: false, ..Default::default() }));
    }

    let track_name = item["name"].as_str().unwrap_or("Unknown").to_string();
    let artist_name = item["artists"][0]["name"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let album_name = item["album"]["name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let album_art_url = item["album"]["images"][0]["url"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let duration_ms = item["duration_ms"].as_u64().unwrap_or(0);
    let progress_ms = body["progress_ms"].as_u64().unwrap_or(0);

    Ok(Some(NowPlaying {
        is_playing,
        track_name,
        artist_name,
        album_name,
        album_art_url,
        duration_ms,
        progress_ms,
    }))
}

/// Play/pause toggle.
pub async fn toggle_playback(access_token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{API_BASE}/me/player/play"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("play request failed: {e}"))?;

    if resp.status().is_success() {
        return Ok(());
    }

    // If play fails (already playing), try pause
    client
        .put(format!("{API_BASE}/me/player/pause"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("pause request failed: {e}"))?;

    Ok(())
}

/// Skip to next track.
pub async fn next_track(access_token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .post(format!("{API_BASE}/me/player/next"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("next request failed: {e}"))?;
    Ok(())
}

/// Skip to previous track.
pub async fn previous_track(access_token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .post(format!("{API_BASE}/me/player/previous"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("previous request failed: {e}"))?;
    Ok(())
}

/// Set playback volume (0-100).
pub async fn set_volume(access_token: &str, volume: u8) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .put(format!("{API_BASE}/me/player/volume"))
        .query(&[("volume_percent", volume.to_string())])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("volume request failed: {e}"))?;
    Ok(())
}

/// Get current playback state (for volume reading).
pub async fn get_playback_state(access_token: &str) -> Result<PlaybackState, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{API_BASE}/me/player"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("playback state request failed: {e}"))?;

    if resp.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(PlaybackState::default());
    }

    if !resp.status().is_success() {
        return Err(format!("playback state failed: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("playback state parse failed: {e}"))?;

    Ok(PlaybackState {
        is_playing: body["is_playing"].as_bool().unwrap_or(false),
        volume: body["device"]["volume_percent"].as_u64().unwrap_or(50) as u8,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub volume: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_code_verifier() {
        let v = generate_code_verifier();
        assert!(v.len() >= 43);
        assert!(v.len() <= 128);
        // Should be base64url (no +, /, =)
        assert!(!v.contains('+'));
        assert!(!v.contains('/'));
        assert!(!v.contains('='));
    }

    #[test]
    fn code_challenge_is_base64url() {
        let v = generate_code_verifier();
        let c = code_challenge(&v);
        assert!(c.len() > 0);
        assert!(!c.contains('+'));
        assert!(!c.contains('/'));
        assert!(!c.contains('='));
    }

    #[test]
    fn generates_unique_state() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert_ne!(s1, s2);
    }

    #[test]
    fn builds_auth_url() {
        let url = auth_url("test_client", "verifier123", "state456");
        assert!(url.contains("client_id=test_client"));
        assert!(url.contains("state=state456"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn tokens_roundtrip_json() {
        let tokens = SpotifyTokens {
            access_token: "abc".into(),
            refresh_token: "def".into(),
            expires_at: 1234567890,
            token_type: "Bearer".into(),
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let back: SpotifyTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(back.access_token, "abc");
        assert_eq!(back.refresh_token, "def");
        assert_eq!(back.expires_at, 1234567890);
    }

    #[test]
    fn now_playing_default_is_empty() {
        let np = NowPlaying::default();
        assert!(!np.is_playing);
        assert!(np.track_name.is_empty());
    }
}