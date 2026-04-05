use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use serenity::http::Http;
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
struct TokenRequestData<'a> {
    client_id: u64,
    client_secret: &'a str,
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: &'a url::Url,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
struct OauthTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

pub async fn login_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    cookies: Cookies,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(cookie) = cookies.get("auth-state") {
        if cookie.value() == params.state {
            let oauth_token: OauthTokenResponse = {
                let form = TokenRequestData {
                    client_id: state.client_id,
                    client_secret: &state.client_secret,
                    grant_type: "authorization_code",
                    code: &params.code,
                    redirect_uri: &state.redirect_uri,
                };

                reqwest::Client::new()
                    .post("https://discord.com/api/oauth2/token")
                    .form(&form)
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to fetch OauthToken: {e:#?}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?
                    .json()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to parse OauthToken response JSON: {e}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?
            };

            let http = Http::new(&format!("Bearer {}", oauth_token.access_token));
            let current_user = http.get_current_user().await.map_err(|e| {
                tracing::error!("failed to fetch current user: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let user = User {
                user_id: current_user.id.get(),
                username: current_user.name.to_owned(),
            };

            state
                .user_storage
                .insert(&user)
                .expect("failed to insert user");

            let session_token = state.token_storage.create(user);

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
