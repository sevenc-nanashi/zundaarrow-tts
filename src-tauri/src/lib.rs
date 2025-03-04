use tokio::sync::Mutex;
use tracing::{error, info};
mod server;
use std::sync::Arc;

static PORT: u16 = 2440;

static ZUNDAMON_SPEECH_SERVER: std::sync::Mutex<Option<server::ZundamonSpeechServer>> =
    std::sync::Mutex::new(None);

#[allow(clippy::type_complexity)]
static WEB_MESSAGE: std::sync::LazyLock<(
    Arc<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>,
    Mutex<tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>>,
)> = std::sync::LazyLock::new(|| {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    (Arc::new(sender), Mutex::new(receiver))
});

pub async fn send_web_message<T: serde::Serialize>(message: T) {
    let message = serde_json::to_value(message).unwrap();
    let (sender, _) = &*WEB_MESSAGE;
    let _ = sender.send(message);
}

#[tauri::command]
async fn open_folder() -> Result<(), String> {
    let path = process_path::get_executable_path().unwrap();
    let path = path.parent().unwrap();
    info!("Opening folder: {:?}", path);
    let _ = open::that(path);
    Ok(())
}

#[tauri::command]
async fn poll_message() -> Result<Option<serde_json::Value>, String> {
    let (_, receiver) = &*WEB_MESSAGE;
    let mut receiver = receiver.lock().await;
    match receiver.recv().await {
        Some(message) => Ok(Some(message)),
        None => Ok(None),
    }
}

#[tauri::command]
async fn launch() -> Result<u16, String> {
    info!("Launching...");

    let port = if server::is_port_open(PORT) {
        PORT
    } else {
        match server::available_port() {
            Ok(port) => port,
            Err(e) => return Err(e.to_string()),
        }
    };

    let root = if cfg!(debug_assertions) {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("zundamon_speech")
    } else {
        process_path::get_executable_path()
            .unwrap()
            .parent()
            .unwrap()
            .join("zundamon_speech")
    };

    let server = server::ZundamonSpeechServer::new(port, &root)
        .await
        .map_err(|e| {
            error!("Failed to start server: {}", e);
            e.to_string()
        })?;

    let mut guard = ZUNDAMON_SPEECH_SERVER.lock().unwrap();
    *guard = Some(server);

    Ok(port)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![launch, open_folder, poll_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
