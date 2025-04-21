use crate::stuff::mailer::Email;
use crate::stuff::order::{save_order, Order};
use crate::stuff::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use log::info;
use reqwest::StatusCode;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/state", post(handle_state))
        .route("/order", post(handle_order))
        .with_state(state)
}
async fn handle_state(body: String) -> crate::Result<impl IntoResponse> {
    info!("Got state: {}", body);
    Email.send_state(body).await?;
    Ok((StatusCode::OK, "state sent").into_response())
}

async fn handle_order(
    State(state): State<AppState>,
    Json(order): Json<Order>,
) -> crate::Result<impl IntoResponse> {
    let order_id = save_order(state, order).await?;
    let res = (StatusCode::CREATED, order_id).into_response();
    Ok(res)
}
