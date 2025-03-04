use anyhow::{Context, Result};
use std::{path::Path, sync::Arc};
use tap::prelude::*;
use tokio::io::AsyncBufReadExt;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::send_notification;

pub struct ZundamonSpeechServer {
    port: u16,
    process: Arc<Mutex<tokio::process::Child>>,
}

impl ZundamonSpeechServer {
    #[tracing::instrument]
    pub async fn new(port: u16, root: &Path) -> Result<Self> {
        let webui = root.join("zundamon-speech-webui");
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
                .current_dir(webui)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .tap(|cmd| info!("Starting server: {:?}", cmd))
                .spawn()
                .context("failed to spawn server")?,
        ));

        tokio::spawn({
            let mut process = process.lock().await;
            let mut stdout =
                tokio::io::BufReader::new(process.stdout.take().context("failed to get stdout")?);
            async move {
                loop {
                    let mut line = String::new();
                    if stdout
                        .read_line(&mut line)
                        .await
                        .map_err(|e| {
                            error!("failed to read server stdout: {}", e);
                            0
                        })
                        .unwrap()
                        == 0
                    {
                        break;
                    }
                    info!("server stdout: {}", line.trim());
                }
            }
        });
        tokio::spawn({
            let mut process = process.lock().await;
            let mut stderr =
                tokio::io::BufReader::new(process.stderr.take().context("failed to get stderr")?);
            async move {
                loop {
                    let mut line = String::new();
                    if stderr
                        .read_line(&mut line)
                        .await
                        .map_err(|e| {
                            error!("failed to read server stderr: {}", e);
                            0
                        })
                        .unwrap()
                        == 0
                    {
                        break;
                    }
                    info!("server stderr: {}", line.trim());
                }
            }
        });

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

                        send_notification(crate::ipc::Notification::ServerExit {
                            code: status.code().unwrap_or(1),
                        });
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });

        Ok(Self { port, process })
    }

    pub async fn kill(self) -> Result<()> {
        let mut process = self.process.lock().await;
        process.kill().await.context("failed to kill server")
    }

    pub async fn is_alive(&self) -> bool {
        self.process.lock().await.try_wait().unwrap().is_none()
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
