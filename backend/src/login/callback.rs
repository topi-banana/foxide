use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use tower_cookies::{
    Cookie, Cookies,
    cookie::time::{Duration, OffsetDateTime},
};

use crate::{AppState, login::User};

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct RequestData<'a> {
    client_id: u64,
    client_secret: &'a str,
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: &'a url::Url,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
pub struct OauthTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

#[derive(Debug, Clone, Deserialize)]
#[expect(dead_code)]
struct UserDataResponse {
    id: String,
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    bot: Option<bool>,
    system: Option<bool>,
    mfa_enabled: Option<bool>,
    banner: Option<String>,
    accent_color: Option<u32>,
    locale: Option<String>,
    verified: Option<bool>,
    email: Option<String>,
    flags: Option<u64>,
    premium_type: Option<u8>,
    public_flags: Option<u64>,
    avatar_decoration_data: Option<serde_json::Value>,
    collectibles: Option<serde_json::Value>,
    primary_guild: Option<serde_json::Value>,
}

pub async fn login_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    cookies: Cookies,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(cookie) = cookies.get("auth-state") {
        if cookie.value() == params.state {
            let client = reqwest::Client::new();

            let oauth_token: OauthTokenResponse = {
                let base_url = url::Url::parse("https://discord.com/api/oauth2/token").unwrap();
                let form = RequestData {
                    client_id: state.client_id,
                    client_secret: &state.client_secret,
                    grant_type: "authorization_code",
                    code: &params.code,
                    redirect_uri: &state.redirect_uri,
                };

                let response = client
                    .post(base_url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(&form)
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to fetch OauthToken: {e:#?}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                response.json().await.map_err(|e| {
                    tracing::error!("failed to parse OauthToken response JSON: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
            };

            let userdata: UserDataResponse = {
                let base_url = url::Url::parse("https://discord.com/api/users/@me").unwrap();
                let response = client
                    .get(base_url)
                    .header(
                        "Authorization",
                        format!("Bearer {}", oauth_token.access_token),
                    )
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to fetch UserData: {e:#?}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                response.json().await.map_err(|e| {
                    tracing::error!("failed to parse UserData response JSON: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
            };

            let user = User {
                user_id: userdata.id.parse::<u64>().unwrap(),
                username: userdata.username,
            };

            state.user_storage.insert(user.clone()).await;

            let session_token = state.token_storage.create(user).await;

            cookies.add(
                Cookie::build(("auth-session", session_token))
                    .http_only(true)
                    .expires(OffsetDateTime::now_utc() + Duration::hours(24 * 30))
                    .path("/")
                    .secure(false)
                    .build(),
            );

            if let Some(redirect_cookie) = cookies.get("auth-redirect") {
                let redirect_path = redirect_cookie.value().to_owned();
                cookies.remove(Cookie::build("auth-redirect").path("/").build());
                Ok(Redirect::to(&redirect_path))
            } else {
                Ok(Redirect::to("/"))
            }
        } else {
            todo!();
        }
    } else {
        todo!();
    }
}
