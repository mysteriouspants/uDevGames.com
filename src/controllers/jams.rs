use chrono::{NaiveDateTime, ParseError as DTParseError};
use diesel::Connection;
use rocket::{get, post, uri, State};
use rocket::{
    request::{Form, FromForm},
    response::Redirect,
};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{
    db::DbPool,
    models::ApprovalState,
    template_helpers::{AdminOnly, JamContext},
};
use crate::{
    models::{Jam, RichText},
    template_helpers::AdminOnlyContext,
};

// CREATE   /jams                   -> jam_id           ADMIN ONLY
// GET      /jams/:jam_id/edit      -> Jam              ADMIN ONLY
// UPDATE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams                   -> Vec<Jam>         All jams when admin,
// GET      /jams/:jam_id/:jam_slug -> Jam              otherwise only published
// DELETE   /jams/:jam_id           -> Result<()>       ADMIN ONLY
// GET      /jams/:jam_id/attachments                   find all attachments for
// GET      /jams/:jam_id/:jam_slug/attachments         a jam... probably ignorable
//                                  -> Vec<Attachment>
// CREATE   /jams/:jam_id/attachments                   create an attachment for this jam
//                                  -> Result<Attachment>

/// Creates a new blank jam and immediately redirects to its edit page.
#[post("/jams")]
pub async fn create_jam(
    pool: State<'_, DbPool>,
    _admin_only: AdminOnly,
) -> Result<Redirect, super::HandlerError> {
    let conn = pool.get()?;
    let jam = Jam::create(&conn)?;
    Ok(Redirect::to(uri!(edit_jam: jam.id)))
}

#[derive(Debug, Serialize)]
struct EditJamContext {
    auth: AdminOnlyContext,
    jam: JamContext,
}

/// Renders out a lovely form that you can use to edit the jam.
#[get("/jams/<jam_id>/edit")]
pub async fn edit_jam(
    pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;
    let jam = match Jam::find_by_id(&conn, jam_id)? {
        Some(jam) => jam,
        None => return Err(super::HandlerError::NotFound),
    };

    let rich_text = match RichText::find_by_id(&conn, jam.rich_text_id)? {
        Some(rich_text) => rich_text,
        None => return Err(super::HandlerError::NotFound),
    };

    let context = EditJamContext {
        auth: admin_only.to_context(),
        jam: JamContext::from_model(&conn, &jam, false)?,
    };

    Ok(Template::render("edit_jam", &context))
}

#[derive(Debug, FromForm)]
pub struct JamFormData {
    // id: i32,
    title: String,
    slug: String,
    summary: String,
    // summary_attachment_id to be set by ajax
    // rich_text_id is already set, not changing that through web calls
    rich_text_content: String,
    start_date: String,
    end_date: String,
    approval_state: String,
}

#[post("/jams/<jam_id>", data = "<jam_form_data>")]
pub async fn update_jam(
    pool: State<'_, DbPool>,
    admin_only: AdminOnly,
    jam_id: i32,
    jam_form_data: Form<JamFormData>,
) -> Result<Template, super::HandlerError> {
    let conn = pool.get()?;

    // do operations in a transaction so that all the updates roll back on
    // failure
    let txr =
        conn.transaction::<(Jam, RichText), super::HandlerError, _>(|| {
            let mut jam = match Jam::find_by_id(&conn, jam_id)? {
                Some(jam) => jam,
                None => return Err(super::HandlerError::NotFound),
            };
            let mut rich_text =
                match RichText::find_by_id(&conn, jam.rich_text_id)? {
                    Some(rich_text) => rich_text,
                    None => return Err(super::HandlerError::NotFound),
                };

            jam.title = jam_form_data.title.clone();
            jam.slug = jam_form_data.slug.clone();
            jam.summary = jam_form_data.summary.clone();
            jam.start_date = parse_date(&jam_form_data.start_date)?;
            jam.end_date = parse_date(&jam_form_data.end_date)?;
            jam.approval_state =
                ApprovalState::from_human_str(&jam_form_data.approval_state)?;
            rich_text.content = jam_form_data.rich_text_content.clone();

            jam.update(&conn)?;
            rich_text.update(&conn)?;
            Ok((jam, rich_text))
        })?;

    let context = EditJamContext {
        auth: admin_only.to_context(),
        jam: JamContext::from_model(&conn, &txr.0, false)?,
    };

    Ok(Template::render("edit_jam", &context))
}

fn parse_date(date: &str) -> Result<NaiveDateTime, DTParseError> {
    NaiveDateTime::parse_from_str(
        &format!("{} 00:00:00", date),
        "%Y-%m-%d %H:%M:%S"
    )
}

#[cfg(test)]
mod tests {
    use super::parse_date;

    #[test]
    fn date_parsing() {
        let foo = parse_date("2021-01-01");
        assert!(foo.is_ok());
    }
}
