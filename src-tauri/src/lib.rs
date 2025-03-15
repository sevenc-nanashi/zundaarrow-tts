mod ipc;
mod server;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing::{error, info};

static PORT: u16 = 2440;

static ZUNDAMON_SPEECH_SERVER: std::sync::Mutex<Option<server::ZundamonSpeechServer>> =
    std::sync::Mutex::new(None);

#[allow(clippy::type_complexity)]
static WEB_NOTIFICATION: std::sync::LazyLock<(
    Arc<tokio::sync::mpsc::UnboundedSender<ipc::Notification>>,
    Mutex<tokio::sync::mpsc::UnboundedReceiver<ipc::Notification>>,
)> = std::sync::LazyLock::new(|| {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    (Arc::new(sender), Mutex::new(receiver))
});

pub fn send_notification(message: ipc::Notification) {
    info!("Sending notification: {:?}", message);
    let (sender, _) = &*WEB_NOTIFICATION;
    let _ = sender.send(message);
}

#[tauri::command]
async fn open_app_folder() -> Result<(), String> {
    let path = process_path::get_executable_path().unwrap();
    let path = path.parent().unwrap();
    info!("Opening folder: {:?}", path);
    let _ = open::that(path);
    Ok(())
}

#[tauri::command]
async fn open_log_folder(app: tauri::AppHandle) -> Result<(), String> {
    let path = app.path().app_log_dir().unwrap();
    info!("Opening folder: {:?}", path);
    let _ = open::that(path);
    Ok(())
}

#[tauri::command]
async fn poll_notification() -> Result<Option<ipc::Notification>, String> {
    let (_, receiver) = &*WEB_NOTIFICATION;
    let mut receiver = receiver.lock().await;

    info!("Polling notification...");
    match receiver.recv().await {
        Some(message) => Ok(Some(message)),
        None => Ok(None),
    }
}

#[tauri::command]
async fn app_info() -> Result<serde_json::Value, String> {
    let app_info = serde_json::json!({
        "version": option_env!("ZTS_VERSION").unwrap_or("0.0.0").to_string(),
        "device": option_env!("ZTS_DEVICE").unwrap_or("unknown").to_string(),
        "buildTimestamp": env!("VERGEN_BUILD_TIMESTAMP").to_string(),
        "commitSha": env!("VERGEN_GIT_SHA").to_string(),
        "rustcVersion": env!("VERGEN_RUSTC_SEMVER").to_string(),
    });

    Ok(app_info)
}

#[tauri::command]
async fn launch(app: tauri::AppHandle) -> Result<u16, String> {
    let old_server = {
        let mut guard = ZUNDAMON_SPEECH_SERVER.lock().unwrap();
        guard.take()
    };

    if let Some(old_server) = old_server {
        info!("Killing old server...");
        if old_server.is_alive().await {
            if let Err(e) = old_server.kill().await {
                error!("Failed to kill old server: {}", e);
                return Err(e.to_string());
            }
        }
    }

    info!("Launching...");

    let port = if server::is_port_open(PORT) {
        PORT
    } else {
        info!(
            "Port {} is not available, trying to find another one...",
            PORT
        );
        match server::available_port() {
            Ok(port) => port,
            Err(e) => return Err(e.to_string()),
        }
    };

    let root = dunce::canonicalize(if cfg!(debug_assertions) {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("zundamon-speech")
    } else {
        app.path().resource_dir().unwrap().join("zundamon-speech")
    })
    .map_err(|e| {
        error!("Failed to get root path: {}", e);
        e.to_string()
    })?;

    let server = server::ZundamonSpeechServer::new(port, &root)
        .await
        .map_err(|e| {
            error!("Failed to start server: {}", e);
            e.to_string()
        })?;

    let mut guard = ZUNDAMON_SPEECH_SERVER.lock().unwrap();
    *guard = Some(server);

    info!("Server started on port {}", port);

    Ok(port)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            launch,
            open_app_folder,
            open_log_folder,
            poll_notification
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    if cfg!(debug_assertions) {
        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_ansi(true)
            .with_env_filter("debug")
            .init();
    } else {
        let log_dir = app.path().app_log_dir().unwrap();
        let current_time = chrono::Local::now();
        let log_file = log_dir.join(current_time.format("%Y-%m-%d_%H-%M-%S.log").to_string());

        std::fs::create_dir_all(&log_dir).unwrap();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file)
            .unwrap();

        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_ansi(false)
            .with_writer(file)
            .init();

        info!("Logging to file: {:?}", log_file);
    }

    #[allow(clippy::single_match)]
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::WindowEvent {
            event: tauri::WindowEvent::CloseRequested { .. },
            ..
        } => {
            if let Ok(mut guard) = ZUNDAMON_SPEECH_SERVER.lock() {
                if let Some(server) = guard.take() {
                    if let Err(e) = tauri::async_runtime::block_on(server.kill()) {
                        error!("Failed to kill server: {}", e);
                    }
                    info!("Server killed");
                }
            }
        }

        _ => {}
    });
}
