#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpServer};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_cors::Cors;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod schema;
mod models;
mod handlers;
mod errors;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", format!(
        "{}=DEBUG,actix_web=INFO,actix_server=INFO",
        utils::env_var("APP_NAME")
    ));
    env_logger::init();

    let pool: models::Pool = r2d2::Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(
            utils::env_var("DATABASE_URL")
        )).expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
        .data(pool.clone())
        .wrap(middleware::Logger::default())
        .wrap(Cors::permissive()) // TODO tighten for production
        .wrap(IdentityService::new(
            CookieIdentityPolicy::new(utils::SECRET_KEY.as_bytes())
            .name("auth")
            .path("/")
            // .domain(utils::env_var("COOKIE_DOMAIN").as_str()) // TODO if cross domain
            .max_age(86400)
            .secure(utils::env_var("API_PROTOCOL") == "https") // TODO https
        ))
        .data(web::JsonConfig::default().limit(4096))
        .service(web::scope("/api")
            .service(web::resource("/invite")
                .route(web::post().to(handlers::invite::invite))
            )
            .service(web::resource("/register")
                .route(web::post().to(handlers::register::register))
            )
            .service(web::resource("/auth")
                .route(web::get().to(handlers::auth::get_me))
                .route(web::post().to(handlers::auth::login))
                .route(web::delete().to(handlers::auth::logout))
            )
            .service(web::scope("/app")
                .wrap_fn(|req, srv| {
                    use actix_identity::RequestIdentity;
                    if req.get_identity().is_some() {
                        use actix_service::Service;
                        srv.call(req)
                    } else {
                        use actix_web::{dev, HttpResponse};
                        use futures::future::{ok, Either};
                        Either::Right(ok(dev::ServiceResponse::new(
                            req.into_parts().0,
                            HttpResponse::Unauthorized().finish()
                        )))
                    }
                })
                .configure(auth_protected)
            )
        )
    })
    .bind(utils::env_var("SOCKET_ADDRESS"))?
    .run()
    .await
}

fn auth_protected(cfg: &mut web::ServiceConfig) {
    cfg
    .service(web::resource("/tasks")
        .route(web::get().to(handlers::app::home::home))
        // .route(web::post().to(handlers::app::text::text))
        // .route(web::put().to(handlers::app::clone::clone))
        // .route(web::delete().to(handlers::app::exec::exec))
    )
    .service(web::resource("/task/{tid}")
        // .route(web::get().to(handlers::app::focus::focus))
        // .route(web::put().to(handlers::app::star::star))
    );
}
