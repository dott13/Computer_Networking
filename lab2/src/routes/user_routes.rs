use actix_web::web;
use crate::controllers::user_controller;

pub fn user_routes() -> impl FnOnce(&mut web::ServiceConfig) {
    move |config| {
        config.service(web::scope("/users")
            .route("", web::post().to(user_controller::create_user))
            .route("", web::get().to(user_controller::get_users))
        );
    }
}
