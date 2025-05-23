use crate::config::config;
use crate::stuff::error::Result;
use crate::stuff::message_handler::MessageHandler;
use crate::stuff::route::get_router;
use log::{error, info};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;

pub struct Poller<H: MessageHandler + Clone + Send + Sync + 'static> {
    handler: H,
}

impl<H> Poller<H>
where
    H: MessageHandler + Clone + Send + Sync + 'static,
{
    pub fn new(handler: H) -> Poller<H> {
        Self { handler }
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let port = config().HOOK_PORT;
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            let listener = TcpListener::bind(addr).await.expect("Failed to bind");

            let app = get_router(tx.clone());

            info!("Hook server started on port {}", port);
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await.expect("Axum server error");
        });

        let mut clonned = self.handler.clone();

        tokio::spawn(async move {
            loop {
                let res = clonned.handle_awaits().await;
                if let Err(e) = res {
                    error!("Awaits handler error: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        loop {
            match rx.recv().await {
                Some(msg) => {
                    self.handler.handle(msg).await?;
                }
                None => {
                    info!("Channel closed, shutting down");
                    return Ok(())
                }
            }
        }
    }
}

async fn shutdown_signal() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stuff::message_handler::Handler;
    use crate::stuff::repository::OrderRepository;
    use crate::stuff::transport::WhatsApp;

    #[tokio::test]
    #[ignore]
    async fn test_poll() {
        let transport = WhatsApp::new();
        let repo = OrderRepository::new();
        let handler = Handler::new(repo, transport);
        let res = Poller::new(handler).start_polling().await;

        if let Err(ref e) = res {
            eprintln!("{}", e);
            assert!(res.is_ok());
        }
    }
}
