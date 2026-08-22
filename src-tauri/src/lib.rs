use std::path::PathBuf;

mod core;
mod workspace;
mod editor;


use core::state::IdeState;

#[tauri::command]
fn get_ide_name() -> String {
    "DragonIDE".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(IdeState::default())
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
        .invoke_handler(tauri::generate_handler![get_ide_name, open_folder, read_workspace, read_file, document_count, open_document, save_document, update_document])
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



#[tauri::command]
fn read_workspace(path: String) -> Result<Vec<workspace::filesystem::FileEntry>, String> {
    let path = PathBuf::from(path);

    workspace::filesystem::read_directory(&path)
}


#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
  let path = PathBuf::from(path);
  workspace::filesystem::read_file(&path)
}


#[tauri::command]
fn document_count(
  state: tauri::State<'_, IdeState>,
) -> Result<usize, String> {

  let documents = state.documents.lock().map_err(|error| error.to_string())?;

  Ok(documents.count())
}

#[tauri::command]
fn open_document(
  path: String,
  state: tauri::State<'_, IdeState>,
) -> Result<String, String> {
  
  let path = PathBuf::from(path);

  let content = workspace::filesystem::read_file(&path)?;

  let mut documents = state.documents.lock().map_err(|error| error.to_string())?;

  let document = documents.open(path, content);

  Ok(document.text.clone())
}


#[tauri::command]
fn update_document(
  path: String,
  text: String,
  state: tauri::State<'_, core::state::IdeState>,
) -> Result<(), String> {

  let path = PathBuf::from(path);

  let mut documents = state.documents.lock().map_err(|error| error.to_string())?;

  let document = documents.get_mut(&path).ok_or_else(|| "Document is not open".to_string())?;

  document.set_text(text);

  Ok(())
}

#[tauri::command]
fn save_document(
  path: String,
  state: tauri::State<'_, core::state::IdeState>,
) -> Result<(), String> {

  let path = PathBuf::from(path);

  let mut documents = state.documents.lock().map_err(|error| error.to_string())?;


  documents.save(&path)
}