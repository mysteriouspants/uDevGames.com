use actix_files::Files;
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
    let gh_client = web::Data::new(gh_client());
    let db_pool = web::Data::new(db_pool);
    let gh_credentials = web::Data::new(gh_credentials);
    let attachment_storage = web::Data::new(attachment_storage);
    init_tera();

    HttpServer::new(move || App::new()
            .wrap(Logger::default())
            /* TODO: .wrap(spehss helmet security headers)*/
            /* TODO: it would be nice to conditionally enable secure(true) when
               running in prod */
            .wrap(CookieSession::signed(&secret.as_bytes()).secure(false))
            .app_data(gh_client.clone())
            .app_data(db_pool.clone())
            .app_data(gh_credentials.clone())
            .app_data(attachment_storage.clone())
            .route("/", web::get().to(crate::controllers::homepage::homepage))
            .route("/login", web::get().to(controllers::gh_oauth::login_with_github))
            .route("/gh_callback", web::get().to(controllers::gh_oauth::gh_callback))
            .route("/logout", web::delete().to(controllers::gh_oauth::logout))
            .service(Files::new("/static", "static"))
        )
        .bind(format!("{}:{}", address, port))
        .expect("Could not bind to address/port")
        .run()
        .await
        .unwrap();
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
