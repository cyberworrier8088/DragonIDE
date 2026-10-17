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
        .invoke_handler(tauri::generate_handler![get_ide_name, read_workspace, read_file, create_file, create_directory, rename_entry, delete_entry, document_count, open_document, save_document, update_document, execute_code, execute_terminal_command, stop_terminal_command, terminal_change_dir, stop_code, search_workspace, find_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
fn read_workspace(path: String) -> Result<Vec<workspace::filesystem::FileEntry>, String> {
    let path = PathBuf::from(path);

    workspace::filesystem::read_directory(&path)
}

// search workspace
#[tauri::command]
async fn search_workspace(root: String, query: String, case_sensitive: bool) -> Result<Vec<workspace::search::SearchResult>, String> {

  tauri::async_runtime::spawn_blocking(move || {
    workspace::search::search_workspace(&PathBuf::from(root), &query, case_sensitive)
  }).await.map_err(|e| e.to_string())?

}

// find files by name (quick open)
#[tauri::command]
async fn find_files(root: String, query: String) -> Result<Vec<workspace::search::FileMatch>, String> {

  tauri::async_runtime::spawn_blocking(move || {
    workspace::search::find_files(&PathBuf::from(root), &query)
  }).await.map_err(|e| e.to_string())?
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
async fn execute_code(file_path: String, language: String, code: String) -> serde_json::Value {

  let result = tauri::async_runtime::spawn_blocking(
    move || match language.as_str() {
      "rust" => execute_rust(&file_path, &code),
      "javascript" => execute_javascript(&file_path, &code),
      "python" => execute_python(&file_path, &code),
      _ => (String::new(), format!("Unsupported language: {}", language), false),

    }).await;

    let (stdout, stderr, success) = result.unwrap_or_else(|e| (String::new(), format!("Execution task failed: {}", e), false));

    serde_json::json!({ "stdout": stdout, "stderr": stderr, "success": success })
}



fn execute_rust(file_path: &str, _code: &str) -> (String, String, bool) {


  let binary = std::env::temp_dir().join(format!("dragonide_run_{}{}", std::process::id(), std::env::consts::EXE_SUFFIX));

  let mut compile = command("rustc");
  compile.arg(file_path).arg("-o").arg(&binary);

  let (_, compile_err, compiled) = run_tracked(compile);
  if !compiled {
    return (String::new(), compile_err, false);
  }

  let result = run_tracked(command(&binary));

  let _ = std::fs::remove_file(&binary);
  result
}

fn execute_javascript(file_path: &str, _code: &str) -> (String, String, bool) {

  let mut cmd = command("node");
  cmd.arg(file_path);
  run_tracked(cmd)
}


fn execute_python(file_path: &str, _code: &str) -> (String, String, bool) {

  let python = if cfg!(target_os = "windows") { "python" } else { "python3" };
  let mut cmd = command(python);
  cmd.arg(file_path);
  run_tracked(cmd)
}

// stop runing command
#[tauri::command]
fn stop_code() -> Result<(), String> {

  let pid = RUNNING_PID.lock().unwrap().take();

  let Some(pid) = pid else {

    return Err("Nothing is running".to_string());
  };

  let pid = pid.to_string();
  let stutus = if cfg!(target_os = "windows") {
    command("taskkill").args(["/PID", pid.as_str(), "/T", "/F"]).status()
  } else {
    command("kill").args(["-9", pid.as_str()]).status()
  };

  stutus.map(|_| ()).map_err(|e| format!("Failed to stop: {}", e))
}

static TERMINAL_PID: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

#[derive(Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum TerminalEvent {
  Line { text: String, error: bool },
  Done { code: i32 },
}

#[tauri::command]
async fn execute_terminal_command(command: String, current_dir: String, on_event: tauri::ipc::Channel<TerminalEvent>) -> Result<(), String> {

    tauri::async_runtime::spawn_blocking(move || run_terminal_command(&command, &current_dir, on_event)).await.map_err(|e| e.to_string())?
}

fn run_terminal_command(command_text: &str, current_dir: &str, on_event: tauri::ipc::Channel<TerminalEvent>) -> Result<(), String> {

  use std::process::Stdio;

  let (shell, flag) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };

  let mut cmd = command(shell);
  cmd.arg(flag).arg(command_text).env("PYTHONUNBUFFERED", "1").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

  // own process group, so stop kills the shell and everthing it started
  #[cfg(unix)]
  {
    use std::os::unix::process::CommandExt;
    cmd.process_group();
  }

  let mut child = cmd.spawn().map_err(|e| format!("Failed to run command: {}", e))?;
  *TERMINAL_PID.lock().unwrap() = Some(child.id());

  let stdout = child.stdout.take().unwrap();
  let stderr = child.stderr.take().unwrap();

  // read stderr on its own thread, so a full pipe can't block the other one
  let err_channel = on_event.clone();
  let err_thread = std::thread::spawn(move || send_lines(stderr, true, &err_channel));
  send_lines(stdout, false, &on_event);
  let _ = err_thread.join();

  let status = child.wait();
  *TERMINAL_PID.lock().unwrap() = None;

  let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
  let _ = on_event.send(TerminalEvent::Done { code });
  Ok(())
}

fn send_lines<R: std::io::Read>(reader: R, error: bool, channel: &tauri::ipc::Channel<TerminalEvent>) {

  use std::io::BufRead;

  let mut reader = std::io::BufReader::new(reader);
  let mut buf = Vec::new();

  loop {
    buf.clear();
    match reader.read_until(b'\n', &mut buf) {
      Ok(0) | Err(_) => break,
      Ok(_) => {
        let text = String::from_utf8_lossy(&buf).trim_end_matches(|c| c == '\r' || c == '\n').to_string();
        let _ = channel.send(TerminalEvent::Line { text, error });
      }
    }
  }
}

#[tauri::command]
fn stop_terminal_command() -> Result<(), String> {
  let Some(pid) = *TERMINAL_PID.lock().unwrap() else {
    return Err("Nothing is running".to_string());

  };

  let status = if cfg!(target_os = "windows") {
    command("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).status()
  } else {
    // "-PID" means the whole process group
    command("kill").args(["-9", "--", &format!("-{}", pid)]).status()
  };

  status.map(|_| ()).map_err(|e| format!("Failed to stop: {}", e))
}

#[tauri::command]
fn terminal_change_dir(current_dir: String, target: String) -> Result<String, String> {
  let target = target.trim().trim_matches('"');

  let next = if target == "-" {
    let home = if cfg!(target_os = "windows") { "USERPROFILE" } else { "HOME" };

    std::env::var(home).map(PathBuf::from).map_err(|_| "cd: home folder not found".to_string())?
  } else {
    PathBuf::from(&current_dir).join(target)
  };

  // remove "." and ".." so the path stays clean
  let mut clean = PathBuf::new();
  for part in next.components() {
    match part {
      std::path::Component::CurDir => {}
      std::path::Component::ParentDir => { clean.pop(); }
      other => clean.push(other.as_os_str()),
    }
  }

  if !clean.is_dir() {
    return Err(format!("cd: no suck folder: {}", target));
  }

  Ok(clean.to_string_lossy().to_string())
}


fn command<S: AsRef<std::ffi::OsStr>>(program: S) -> std::process::Command {

  let mut cmd = std::process::Command::new(program);

  #[cfg(target_os = "windows")]
  {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
  }
  cmd
}


static RUNNING_PID: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

fn run_tracked(mut cmd: std::process::Command) -> (String, String, bool) {

  use std::process::Stdio;

  let program = cmd.get_program().to_string_lossy().to_string();

  let child = match cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
    Ok(child) => child,
    Err(e) => return (String::new(), format!("Could not start {}: {}", program, e), false),
  };

  *RUNNING_PID.lock().unwrap() = Some(child.id());
  let result = child.wait_with_output();
  *RUNNING_PID.lock().unwrap() = None;

  match result {
    Ok(output) => (

      String::from_utf8_lossy(&output.stdout).to_string(),
      String::from_utf8_lossy(&output.stderr).to_string(),
      output.status.success(),
    ),
    Err(e) => (String::new(), format!("Failed to run: {}", e), false),
  }
}