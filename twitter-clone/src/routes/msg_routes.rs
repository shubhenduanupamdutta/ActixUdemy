use actix_web::web;

use crate::{common::entities::base::DbRepo, routes::handler::msg_handlers};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/messages")
            .route("", web::post().to(msg_handlers::create_message::<DbRepo>))
            .route("/{id}", web::get().to(msg_handlers::get_message::<DbRepo>))
            .route("/", web::get().to(msg_handlers::get_messages::<DbRepo>)),
    );
}
