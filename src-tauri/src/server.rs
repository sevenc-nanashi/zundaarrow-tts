use anyhow::{Context, Result};
use std::{path::Path, sync::Arc};
use tap::prelude::*;
use tokio::sync::Mutex;
use tracing::info;

pub struct ZundamonSpeechServer {
    port: u16,
    process: Arc<Mutex<tokio::process::Child>>,
}

impl ZundamonSpeechServer {
    #[tracing::instrument]
    pub async fn new(port: u16, root: &Path) -> Result<Self> {
        let webui = root.join("webui");
        let server = root.join("server").join("main.py");
        let python = root
            .join("standalone_python")
            .join("bin")
            .join(if cfg!(windows) {
                "python.exe"
            } else {
                "python3"
            });

        let process = Arc::new(Mutex::new(
            tokio::process::Command::new(python)
                .arg(server)
                .arg(port.to_string())
                .env("VIRTUAL_ENV", webui.join("venv"))
                .tap(|cmd| info!("Starting server: {:?}", cmd))
                .spawn()
                .context("failed to spawn server")?,
        ));

        tokio::spawn({
            let process = Arc::clone(&process);
            async move {
                loop {
                    if let Some(status) = {
                        process
                            .lock()
                            .await
                            .try_wait()
                            .context("failed to check server status")
                            .ok()
                            .flatten()
                    } {
                        info!("server exited with status: {}", status);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });

        for _ in 0..300 {
            info!("Checking if port {} is open", port);
            if !is_port_open(port) {
                info!("Server started on port {}", port);
                return Ok(Self { port, process });
            }
            if let Some(status) = {
                process
                    .lock()
                    .await
                    .try_wait()
                    .context("failed to check server status")
                    .ok()
                    .flatten()
            } {
                anyhow::bail!("server exited with status: {}", status);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        let mut process = process.lock().await;
        process.kill().await.context("failed to kill server")?;

        anyhow::bail!("failed to start server");
    }
}

pub fn is_port_open(port: u16) -> bool {
    std::net::TcpListener::bind(("localhost", port)).is_ok()
}

pub fn available_port() -> std::io::Result<u16> {
    match std::net::TcpListener::bind("localhost:0") {
        Ok(listener) => Ok(listener.local_addr().unwrap().port()),
        Err(e) => Err(e),
    }
}
