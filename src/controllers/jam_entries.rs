use rocket::{State, post, response::Redirect};
use rocket_contrib::templates::Template;

use crate::{db::DbPool, template_helpers::UserRequired};

// CREATE   /jams/:jam_id/entries               -> jam_entry_id     USERS ONLY
// UPDATE   /jams/:jam_id/entries/:jam_entry_id -> Result<()>       ADMIN/OWNER ONLY
// marking a jam as published is admin-only.
// GET      /jams/:jam_id/:jam_slug/entries     -> Vec<JamEntries>  All when admin,
// GET      /jams/:jam_id/:jam_slug/:jam_entry_id/:jam_entry_slug   otherwise only
//                                              -> Jam              published
// DELETE   /jams/:jam_id/entries/:jam_entry_id -> Result<()>       ADMIN ONLY

#[post("/jams/<jam_id>/entries")]
pub async fn create_jam_entry(
    pool: State<'_, DbPool>,
    user: UserRequired,
    jam_id: i32,
) -> Result<Redirect, super::HandlerError> {
    // user must not be banned

    // user must not already have a jam entry; if they do, can we just redirect
    // them to that entry?

    // the jam must be active
}