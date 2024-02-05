use color_eyre::eyre::Result;
use crossterm::cursor::Show;
use crossterm::execute;
use std::io::stdout;
use std::{
    error::Error,
    fmt,
    fmt::Display,
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::{signal, sync::broadcast};

#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyCreatedError;

impl Error for AlreadyCreatedError {}

impl Display for AlreadyCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("shutdown handler already created")
    }
}

static CREATED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct Shutdown {
    pub sender: broadcast::Sender<()>,
}

impl Shutdown {
    pub fn new() -> Result<Self, AlreadyCreatedError> {
        if (CREATED).swap(true, Ordering::SeqCst) {
            log::error!("shutdown handler called twice");
            return Err(AlreadyCreatedError);
        }

        let (tx, _) = broadcast::channel(1);
        let handle = register_handlers();

        let tx_for_handle = tx.clone();
        tokio::spawn(async move {
            log::debug!("Registered shutdown handlers");
            handle.await;
            tx_for_handle.send(()).ok();
        });

        Ok(Self { sender: tx })
    }

    pub fn handle(&self) -> impl Future<Output = ()> + '_ {
        let mut rx = self.sender.subscribe();

        async move {
            let rx = rx.recv();

            rx.await.unwrap();
        }
    }
}

fn register_handlers() -> impl Future<Output = ()> {
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

    async move {
        tokio::select! {
            _ = ctrl_c => {
                log::info!("Received Ctrl+C signal");
                if let Err(e) = execute!(stdout(), Show) {
                    eprintln!("Failed to restore cursor: {e}");
                }
            },
            _ = terminate => {
                log::info!("Received terminate signal");
                if let Err(e) = execute!(stdout(), Show) {
                    eprintln!("Failed to restore cursor: {e}");
                }
            },
        }
    }
}
