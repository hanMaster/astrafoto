use log::info;
use tokio::signal;

pub mod config;
pub mod state;
pub mod routes;
pub mod order;
pub mod mailer;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT."),
        _ = terminate => info!("Received SIGTERM."),
    }

    info!("signal received, starting graceful shutdown");
}