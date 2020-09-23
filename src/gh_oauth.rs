use crate::db::DbPool;
use crate::models::GhUserRecord;
use reqwest::Client as ReqwestClient;
use rocket::{get, State};
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};
use thiserror::Error;


pub struct GhCredentials {
    pub client_id: String,
    pub client_secret: String,
}

pub fn gh_client() -> ReqwestClient {
    reqwest::ClientBuilder::new()
        // github requires that a user agent be set to use its api
        .user_agent("Rust/reqwest/uDevGames.com")
        .build()
        .unwrap()
}

#[get("/login_with_github")]
pub fn login_with_github(gh_credentials: State<GhCredentials>) -> Template {
    #[derive(Serialize)]
    struct Context {
        oauth_url: String
    };

    let context = Context {
        oauth_url: format!(
            "http://github.com/login/oauth/authorize?client_id={}",
            gh_credentials.client_id
        )
    };

    Template::render("login_with_github", &context)
}

#[get("/gh_callback?<code>")]
pub async fn gh_callback(
    gh_credentials: State<'_, GhCredentials>,
    gh_client: State<'_, ReqwestClient>,
    db_pool: State<'_, DbPool>,
    code: String
) -> Template {
    auth_with_github(&gh_client, &db_pool, &gh_credentials, &code).await;

    #[derive(Serialize)]
    struct Context {

    };

    let context = Context { };

    Template::render("gh_callback", &context)
}

#[derive(Deserialize, Debug)]
struct AuthorizationResponse {
    access_token: String, token_type: String, scope: String,
}

async fn auth_with_github(
    gh_client: &ReqwestClient, db_pool: &DbPool,
    gh_credentials: &GhCredentials, code: &String
) {
    let authorization = get_access_token(&gh_client, &gh_credentials, &code).await;
    println!("authorization: {:?}", authorization);
    let user = get_or_update_user_detail(
        &gh_client, &db_pool, &authorization
    ).await;
    println!("user: {:?}", user);
}

async fn get_access_token(
    gh_client: &ReqwestClient, gh_credentials: &GhCredentials, code: &String
) -> AuthorizationResponse {
    let params = [
        ("client_id", gh_credentials.client_id.as_str()),
        ("client_secret", gh_credentials.client_secret.as_str()),
        ("code", code.as_str())
    ];

    gh_client.post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// Broadly speaking, these are the only fields we're truly interested in from
/// Github. The id is the most important, for it is how we can durably refer to
/// a user even if they change their alias on Github. The login pre-populates
/// a user's identity on uDevGames, and the avatar and link to their github
/// might become useful in the future, though it's not a sure thing.
#[derive(Deserialize, Debug)]
struct UserResponse {
    login: String, id: i64, avatar_url: String, html_url: String
}

#[derive(Error, Debug)]
enum GetOrUpdateUserDetailError {
    #[error("Problem loading data. Call a DBA.")]
    DbLoadError(#[from] crate::models::ModelError)
}

async fn get_or_update_user_detail(
    gh_client: &ReqwestClient,
    db_pool: &DbPool,
    authorization: &AuthorizationResponse
) -> Result<GhUserRecord, GetOrUpdateUserDetailError> {
    let user = get_user_detail(
        &gh_client, &authorization.access_token
    ).await;
    let gh_user_record = GhUserRecord::find_and_update(
        &db_pool, user.id, &user.login, &user.avatar_url, &user.html_url
    )?;

    Ok(gh_user_record)
}

async fn get_user_detail(
    gh_client: &ReqwestClient, access_token: &String
) -> UserResponse {
   gh_client.get("https://api.github.com/user")
        .header("Authorization", format!("token {}", access_token))
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
