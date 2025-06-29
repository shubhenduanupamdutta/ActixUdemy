use actix_web::web;

use crate::{common::entities::base::DbRepo, routes::handler::profile_handlers};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/profile")
            .route(
                "/{id}",
                web::get().to(profile_handlers::get_profile::<DbRepo>),
            )
            .route(
                "/",
                web::post().to(profile_handlers::create_profile::<DbRepo>),
            )
            .route(
                "/username/{user_name}",
                web::get().to(profile_handlers::get_profile_by_user_name::<DbRepo>),
            ),
    );
}
