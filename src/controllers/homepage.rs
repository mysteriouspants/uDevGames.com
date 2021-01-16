use crate::models::Jam;
use crate::{
    application_context::ApplicationContext,
    template_helpers::{JamContext, UserOptional, UserOptionalContext},
};
use actix_session::Session;
use actix_web::{http::header::ContentType, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    show_all_jams: Option<bool>,
}

// GET /?[show_all_jams=true|false]
pub async fn homepage(
    ctxt: web::Data<ApplicationContext>,
    session: Session,
    query_params: web::Query<QueryParams>,
) -> Result<HttpResponse, super::HandlerError> {
    let conn = ctxt.db_pool.get()?;
    let user = UserOptional::from_session(&conn, &session)?;

    let should_show_all_jams =
        user.is_admin() && query_params.show_all_jams.unwrap_or(false);

    // load the first three approved jams
    let mut jams = Vec::new();
    let can_create_new_jam_entries = !user.is_banned();

    for j in Jam::find_all(&conn, !should_show_all_jams, 0, 3)? {
        jams.push(JamContext::from_model(&conn, &j, false)?);
    }

    #[derive(Debug, Serialize)]
    struct Context {
        auth: UserOptionalContext,
        jams: Vec<JamContext>,
        showing_all_jams: bool,
        can_create_new_jam_entries: bool,
    }

    let context = Context {
        jams,
        auth: user.to_context(),
        showing_all_jams: should_show_all_jams,
        can_create_new_jam_entries,
    };

    Ok(HttpResponse::Ok()
        .set(ContentType::html())
        .body(ctxt.render_template("homepage.html.tera", &context)))
}
