use axum::{
    extract::{Query, State},
    response::Redirect,
};
use serde::Deserialize;
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use crate::AppState;

#[derive(Deserialize)]
pub struct LoginParms {
    redirect: Option<String>,
}

fn create_invite_url(client_id: u64, redirect_uri: &url::Url, state: &str) -> url::Url {
    let mut base_url = url::Url::parse("https://discord.com/oauth2/authorize").unwrap();

    base_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_id.to_string())
        .append_pair("redirect_uri", redirect_uri.as_str())
        .append_pair("state", state)
        .append_pair("scope", "identify");
    base_url
}

pub async fn login_redirect(
    State(state): State<AppState>,
    Query(params): Query<LoginParms>,
    cookies: Cookies,
) -> Redirect {
    use rand::distr::{Alphanumeric, SampleString};

    let auth_state = Alphanumeric.sample_string(&mut rand::rng(), 32);

    let url = create_invite_url(state.client_id, &state.redirect_uri, &auth_state);

    cookies.add(
        Cookie::build(("auth-state", auth_state))
            .http_only(true)
            .max_age(Duration::minutes(10))
            .path("/")
            .same_site(SameSite::Lax)
            .secure(false)
            .build(),
    );

    if let Some(redirect_path) = params.redirect {
        cookies.add(
            Cookie::build(("auth-redirect", redirect_path))
                .http_only(true)
                .max_age(Duration::minutes(10))
                .path("/")
                .same_site(SameSite::Lax)
                .secure(false)
                .build(),
        );
    }

    Redirect::to(url.as_str())
}
