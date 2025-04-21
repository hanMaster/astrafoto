use crate::stuff::routes::get_router;
use crate::stuff::state::AppState;
pub use errors::Result;
use std::net::SocketAddr;
use log::info;
use tokio::net::TcpListener;

mod errors;
mod stuff;

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState::new()?;
    pretty_env_logger::init_timed();
    let port = state.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    let app = get_router(state);
    info!("Server started on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}







