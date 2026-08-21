use std::path::PathBuf;

mod core;
mod workspace;

#[tauri::command]
fn get_ide_name() -> String {
    "DragonIDE".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_ide_name, open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



#[tauri::command]
async fn open_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {

  let folder = tauri_plugin_dialog::DialogExt::dialog(&app).file().blocking_pick_folder();

  match folder {

    Some(path) => {

      let path: PathBuf = path.into_path().map_err(|e| e.to_string())?;

      Ok(Some(path.to_string_lossy().to_string()))
    }

    None => Ok(None),
  }
}