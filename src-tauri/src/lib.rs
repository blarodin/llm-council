mod api;
mod config;
mod council;
mod models;
mod openrouter;
mod storage;

use std::net::SocketAddr;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Load environment variables from .env file
  dotenv::dotenv().ok();

  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Start Axum server in a separate tokio runtime
      std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        runtime.block_on(async {
          start_axum_server().await;
        });
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

async fn start_axum_server() {
  let app = api::create_router();

  // Bind to 127.0.0.1:8001 (same port as Python version)
  let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
  
  println!("🚀 Axum server running on http://{}", addr);

  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Failed to bind to port 8001");

  axum::serve(listener, app)
    .await
    .expect("Failed to start Axum server");
}
