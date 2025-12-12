use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    controllers::poll_handlers::{
        close_poll_handler::close_poll_handler::close_poll_handler,
        create_poll_handler::create_poll_handler::create_poll_handler,
        get_stats_of_particular_poll_handler::get_poll_handler::get_poll,
        list_polls_handler::list_polls::list_polls,
        particular_user_polls_handler::user_polls::user_polls,
        poll_reset_handler::poll_reset_handler::reset_poll_handler,
        poll_results_sse_handler::poll_results_sse_handler::poll_results_sse_handler,
        poll_vote_handler::poll_vote_handler::vote_handler,
    },
    middleware::authentication_middleware::{optional_authentication, require_authentication},
    models::local_store::AppState,
};

pub fn polls_routes() -> Router<AppState> {
    // Public routes with optional auth
    let public_routes = Router::new()
        .route("/", get(list_polls))
        .route("/{poll_id}", get(get_poll))
        .route("/{poll_id}/results", get(poll_results_sse_handler))
        .route_layer(middleware::from_fn(optional_authentication));

    // Protected routes requiring auth
    let protected_routes = Router::new()
        .route("/new", post(create_poll_handler))
        .route("/{poll_id}/vote", post(vote_handler))
        .route("/{poll_id}/reset", post(reset_poll_handler))
        .route("/{poll_id}/close", post(close_poll_handler))
        .route("/user/{user_id}", get(user_polls))
        .route_layer(middleware::from_fn(require_authentication));

    // Merge both route groups
    Router::new().merge(public_routes).merge(protected_routes)
}
