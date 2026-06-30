//! Turbo Sport database editor backend. Commands to load, validate, and save a starting
//! database (the football crate owns the `Database` type and its file/validation logic).

use football::Database;

/// Return the built-in sample database to start editing from.
#[tauri::command]
fn load_sample() -> Database {
    Database::sample()
}

/// Open a database from a JSON file.
#[tauri::command]
fn open(path: String) -> Result<Database, String> {
    football::database::load(&path).map_err(|e| e.to_string())
}

/// Validate then write a database to a JSON file.
#[tauri::command]
fn save(path: String, db: Database) -> Result<(), String> {
    db.validate()?;
    football::database::save(&db, &path).map_err(|e| e.to_string())
}

/// Validate a database without writing it (referential integrity).
#[tauri::command]
fn validate(db: Database) -> Result<(), String> {
    db.validate()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![load_sample, open, save, validate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
