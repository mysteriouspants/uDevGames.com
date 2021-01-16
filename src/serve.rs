use actix_files::Files;
use actix_session::CookieSession;
use actix_web::{middleware::Logger, web, App, HttpServer};

use crate::{
    application_context::ApplicationContext,
    attachments::AttachmentStorage,
    controllers::{
        self,
        gh_oauth::{gh_client, GhCredentials},
    },
    db::DbPool,
    view::init_tera,
};

pub async fn serve(
    address: String,
    port: u16,
    secret: String,
    db_pool: DbPool,
    gh_credentials: GhCredentials,
    attachment_storage: AttachmentStorage,
) {
    let tera = init_tera();
    let application_context = web::Data::new(ApplicationContext {
        gh_client: gh_client(),
        gh_credentials,
        db_pool,
        attachment_storage,
        tera,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            /* TODO: .wrap(spehss helmet security headers)*/
            /* TODO: it would be nice to conditionally enable secure(true) when
               running in prod */
            .wrap(CookieSession::signed(&secret.as_bytes()).secure(false))
            .app_data(application_context.clone())
            .route("/", web::get().to(crate::controllers::homepage::homepage))
            .route(
                "/login",
                web::get().to(controllers::gh_oauth::login_with_github),
            )
            .route(
                "/gh_callback",
                web::get().to(controllers::gh_oauth::gh_callback),
            )
            .route("/logout", web::delete().to(controllers::gh_oauth::logout))
            .service(Files::new("/static", "static"))
    })
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
}
