use actix_session::CookieSession;
use actix_web::{App, HttpServer, middleware::Logger, web};
use tera::Tera;

use crate::{
    attachments::AttachmentStorage, controllers::gh_oauth::{ gh_client, GhCredentials },
    db::DbPool,
};

pub async fn serve(
    address: String,
    port: u16,
    secret: String,
    db_pool: DbPool,
    gh_credentials: GhCredentials,
    attachment_storage: AttachmentStorage,
) {
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error parsing template: {}", e);
            std::process::exit(-1);
        }
    };

    HttpServer::new(|| App::new()
            .wrap(Logger::default())
            /* TODO: it would be nice to conditionally enable secure(true) when
               running in prod */
            /* TODO: .wrap(spehss helmet security headers)*/
            .wrap(CookieSession::signed(secret.bytes()).secure(false))
            .data(tera)
            .data(gh_client())
            .data(db_pool)
            .data(gh_credentials)
            .data(attachment_storage)
            .route("/", web::get().to(crate::controllers::homepage::homepage))
        )
        .bind(format!("{}:{}", address, port))
        .run()
        .await;
    //     .mount(
    //         "/",
    //         routes![
    //             crate::controllers::attachments::get_attachment,
    //             crate::controllers::gh_oauth::login_with_github,
    //             crate::controllers::gh_oauth::gh_callback,
    //             crate::controllers::gh_oauth::logout,
    //             crate::controllers::jams::create_jam,
    //             crate::controllers::jams::edit_jam,
    //             crate::controllers::jams::update_jam,
    //         ],
    //     )
    //     .mount("/static", StaticFiles::from(crate_relative!("/static")))
}
