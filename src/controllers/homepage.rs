use crate::template_helpers::{JamContext, UserOptional, UserOptionalContext};
use crate::{db::DbPool, models::Jam};
use crate::view::tera;
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::ContentType, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    show_all_jams: Option<bool>
}

// GET /?[show_all_jams=true|false]
pub fn homepage(
    pool: web::Data<DbPool>,
    session: Session,
    query_params: web::Query<QueryParams>,
    req: HttpRequest,
) -> Result<impl Responder, super::HandlerError> {
    let conn = pool.get()?;
    let user = UserOptional::from_session(conn, session)?;

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

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(tera().render("homepage", &context)?)
    )
}
