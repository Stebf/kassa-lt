mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Initialize database
      if let Err(e) = commands::init_db(app.handle()) {
        eprintln!("Failed to initialize database: {}", e);
      }
      
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_products,
      commands::add_product,
      commands::checkout,
      commands::get_orders,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
