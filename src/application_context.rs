use reqwest::Client as ReqwestClient;
use serde::Serialize;
use tera::{Context, Tera};

use crate::attachments::AttachmentStorage;
use crate::controllers::gh_oauth::GhCredentials;
use crate::db::DbPool;

pub struct ApplicationContext {
    pub gh_client: ReqwestClient,
    pub gh_credentials: GhCredentials,
    pub db_pool: DbPool,
    pub attachment_storage: AttachmentStorage,
    pub tera: Tera, // both send and sync, so no additional wrapping required
}

impl ApplicationContext {
    pub fn render_template(
        &self,
        tname: &str,
        ctxt: &impl Serialize,
    ) -> String {
        self.tera
            .render(tname, &Context::from_serialize(ctxt).unwrap())
            .expect("Could not render template")
    }
}
