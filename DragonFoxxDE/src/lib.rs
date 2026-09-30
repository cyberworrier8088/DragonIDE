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
        .invoke_handler(tauri::generate_handler![get_ide_name, read_workspace, read_file, create_file, create_directory, rename_entry, delete_entry, document_count, open_document, save_document, update_document, execute_code, execute_terminal_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
fn create_file(path: String) -> Result<(), String> {

  let path = PathBuf::from(path);

  workspace::filesystem::create_file(&path)
}

#[tauri::command]
fn create_directory(path: String) -> Result<(), String> {

  let path = PathBuf::from(path);

  workspace::filesystem::create_directory(&path)
}

#[tauri::command]
fn rename_entry(old_path: String, new_path: String) -> Result<(), String> {
    let old_path = PathBuf::from(old_path);
    let new_path = PathBuf::from(new_path);
    workspace::filesystem::rename_entry(&old_path, &new_path)
}

#[tauri::command]
fn delete_entry(path: String, is_directory: bool) -> Result<(), String> {
    let path = PathBuf::from(path);
    workspace::filesystem::delete_entry(&path, is_directory)
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


#[tauri::command]
fn execute_code(file_path: String,language: String,code: String) -> serde_json::Value {


  let (stdout, stderr, success) = match language.as_str() {
    
    "rust" => execute_rust(&file_path, &code),
    "javascript" => execute_javascript(&file_path, &code),
    "python" => execute_python(&file_path, &code),
    _ => (
      String::new(),
      format!("Unsupported language: {}", language),
      false
    )
  };

  serde_json::json!({
    "stdout": stdout,
    "stderr": stderr,
    "success": success
  })
}

fn execute_rust(file_path: &str, _code: &str) -> (String, String, bool) {

  use std::process::Command;

  let compile_output = Command::new("rustc").arg(file_path).arg("-o").arg("./temp_rust_binary").output();


  match compile_output {
    Ok(output) => {
      if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return (String::new(), stderr, false);
      }

      let run_output = Command::new("./temp_rust_binary").output();


      match run_output {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout).to_string();
          let stderr = String::from_utf8_lossy(&output.stderr).to_string();
          let _ = std::fs::remove_file("./temp_rust_binary");

          (stdout, stderr, output.status.success())
        }
        Err(e) => {
          (String::new(), format!("Failed to run: {}", e), false)
        }
      }
    }

    Err(e) => {
      (String::new(), format!("Compilation failed: {}", e), false)
    }
  }
}


fn execute_javascript(file_path: &str, _code: &str) -> (String, String, bool) {
  use std::process::Command;

  let output = Command::new("node").arg(file_path).output();

  match output {
    Ok(output) => {
      let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();
      (stdout, stderr, output.status.success())
    }
    Err(e) => {
      (String::new(), format!("Node.js not found or error: {}", e), false)
    }
  }
}

fn execute_python(file_path: &str, _code: &str) -> (String, String, bool) {
  use std::process::Command;

  let output = Command::new("python3").arg(file_path).output();

  match output {
    Ok(output) => {
      let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();
      (stdout, stderr, output.status.success())
    }
    Err(e) => {
      (String::new(), format!("Python3 not found or error: {}", e), false)
    }
  }
}


#[tauri::command]
fn execute_terminal_command(command: String, current_dir: String) -> Result<String, String> {

  use std::process::Command;

  let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
  let arg = if cfg!(target_os = "windows") { "/c" } else { "-c" };

  let output = Command::new(shell).arg(arg).arg(&command).current_dir(current_dir).output();

  match output {
    Ok(output) => {

      let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();

      if !output.status.success() {

        Ok(stderr)
      } else {
        let mut combined = stdout;

        if !stderr.is_empty() {

          combined.push_str("\n");
          combined.push_str(&stderr);
        }

        Ok(combined)
      }
    }

    Err(e) => Err(format!("Failed to run command: {}", e)),
  }
}