use actix_session::CookieSession;
use actix_web::{App, HttpServer, middleware::Logger, web};

use crate::{attachments::AttachmentStorage, controllers::{self, gh_oauth::{ gh_client, GhCredentials }}, db::DbPool, view::init_tera};

pub async fn serve(
    address: String,
    port: u16,
    secret: String,
    db_pool: DbPool,
    gh_credentials: GhCredentials,
    attachment_storage: AttachmentStorage,
) {
    let gh_client = gh_client();
    init_tera();

    HttpServer::new(|| App::new()
            .wrap(Logger::default())
            /* TODO: .wrap(spehss helmet security headers)*/
            /* TODO: it would be nice to conditionally enable secure(true) when
               running in prod */
            .wrap(CookieSession::signed(secret.bytes()).secure(false))
            .data(gh_client)
            .data(db_pool)
            .data(gh_credentials)
            .data(attachment_storage)
            .route("/", web::get().to(controllers::homepage::homepage))
            .route("/login", web::get().to(controllers::gh_oauth::login_with_github))
            .route("/gh_callback", web::get().to(controllers::gh_oauth::gh_callback))
            .route("/logout", web::delete().to(controllers::gh_oauth::logout))
        )
        .bind(format!("{}:{}", address, port))
        .run()
        .await;
    //     .mount(
    //         "/",
    //         routes![
    //             crate::controllers::attachments::get_attachment,
    //             crate::controllers::jams::create_jam,
    //             crate::controllers::jams::edit_jam,
    //             crate::controllers::jams::update_jam,
    //         ],
    //     )
    //     .mount("/static", StaticFiles::from(crate_relative!("/static")))
}
