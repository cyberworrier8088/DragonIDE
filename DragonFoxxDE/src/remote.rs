// Remote mode: the laptop serves DragonIDE on WIFI, so a phone browser can use it.
// The phone only shows the UI. Failes, git and code all stay on the laptop

use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

const PORT: u16 = 7777;
const MAX_BODY: u64 = 10 * 1024 * 1024; // nothing bigger than 10MB comes from a phone
const MAX_TERMINAL_LINES: usize = 10_000;

struct Remote {
    app: AppHandle,
    token: String,
    root: PathBuf,
    allow_run: bool,
    // terminal lwaiting for the phone to pick them up
    terminal: Arc<Mutex<Vec<crate::TerminalEvent>>>,
}

#[derive(Default)]
pub struct RemoteServer {
    server: Mutex<Option<Arc<Server>>>,
}

#[derive(serde::Serialize)]
pub struct RemoteInfo {
    url: String,
    qr_svg: String,
}

#[tauri::command]
pub fn start_remote(
    app: AppHandle,
    workspace: String,
    allow_run: bool,
    server: tauri::State<'_, RemoteServer>,
) -> Result<RemoteInfo, String> {
    let root = PathBuf::from(&workspace)
        .canonicalize()
        .map_err(|_| "Open a folder first".to_string())?;

    stop_server(&server);

    let http =
        Server::http(("0.0.0.0", PORT)).map_err(|e| format!("Port {} is busy: {}", PORT, e))?;
    let http = Arc::new(http);

    *server.server.lock().unwrap() = Some(http.clone());

    let token = new_token()?;
    let remote = Arc::new(Remote {
        app,
        token: token.clone(),
        root,
        allow_run,
        terminal: Arc::default(),
    });

    std::thread::spawn(move || {
        for request in http.incoming_requests() {
            let remote = remote.clone();
            std::thread::spawn(move || handle(&remote, request));
        }
    });

    let url = format!("http://{}:{}/?token={}", laptop_ip(), PORT, token);
    let qr_svg = qrcode::QrCode::new(&url)
        .map_err(|e| e.to_string())?
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(180, 180)
        .build();

    Ok(RemoteInfo { url, qr_svg })
}

#[tauri::command]
pub fn stop_remote(server: tauri::State<'_, RemoteServer>) {
    stop_server(&server);
}

fn stop_server(server: &RemoteServer) {
    if let Some(http) = server.server.lock().unwrap().take() {
        http.unblock();
    }
}

fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
}

fn laptop_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|address| address.ip().to_string())
        .unwrap_or_else(|_| "localhost".to_string())
}

fn handle(remote: &Remote, mut request: Request) {
    let url = request.url().to_string();

    let response = match (request.method(), url.strip_prefix("/api/")) {
        (Method::Post, Some(command)) => {
            let (status, body) = api(remote, command, &mut request);
            json_response(status, body)
        }
        (Method::Get, _) => page(remote, &url),
        _ => Response::from_data(b"Not found".to_vec()).with_status_code(404),
    };

    let _ = request.respond(response);
}

fn page(remote: &Remote, url: &str) -> Response<Cursor<Vec<u8>>> {
    let path = url.split('?').next().unwrap_or("/");

    match remote.app.asset_resolver().get(path.to_string()) {
        Some(asset) => {
            Response::from_data(asset.bytes).with_header(header("Content-Type", &asset.mime_type))
        }
        None => Response::from_data(b"No found".to_vec()).with_status_code(404),
    }
}

fn api(remote: &Remote, command: &str, request: &mut Request) -> (u16, Value) {
    let sent = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Authorization"))
        .map(|h| h.value.as_str().to_string())
        .unwrap_or_default();

    if sent != format!("Bearer {}", remote.token) {
        return (
            401,
            json!({ "error": "Wrong or old link. Scan the QR code again." }),
        );
    }

    let mut body = String::new();

    if request
        .as_reader()
        .take(MAX_BODY)
        .read_to_string(&mut body)
        .is_err()
    {
        return (400, json!({ "error": "Could not read the request" }));
    }

    let args: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    match tauri::async_runtime::block_on(run(remote, command, &args)) {
        Ok(result) => (200, json!({ "result": result })),
        Err(error) => (400, json!({ "error": error })),
    }
}

fn json_response(status: u16, body: Value) -> Response<Cursor<Vec<u8>>> {
    Response::from_data(body.to_string().into_bytes())
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json"))
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

async fn run(remote: &Remote, command: &str, args: &Value) -> Result<Value, String> {
    let text = |name: &str| -> Result<String, String> {
        args.get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("Missing \"{}\"", name))
    };

    let flag = |name: &str| args.get(name).and_then(Value::as_bool).unwrap_or(false);

    let path = |name: &str| -> Result<String, String> {
        let path = text(name)?;
        if inside(&remote.root, Path::new(&path)) {
            Ok(path)
        } else {
            Err("That path is outside the opened folder".to_string())
        }
    };

    let state = remote.app.state::<crate::IdeState>();

    let result = match command {
        "get_ide_name" => json!(crate::get_ide_name()),
        "remote_workspace" => json!(remote.root.to_string_lossy()),
        "read_workspace" => json!(crate::read_workspace(path("path")?)?),
        "read_file" => json!(crate::read_file(path("path")?)?),
        "create_file" => json!(crate::create_file(path("path")?)?),
        "create_directory" => json!(crate::create_directory(path("path")?)?),
        "rename_entry" => json!(crate::rename_entry(path("oldPath")?, path("newPath")?)?),
        "delete_entry" => json!(crate::delete_entry(path("path")?, flag("isDirectory"))?),
        "paste_entry" => json!(crate::paste_entry(
            path("from")?,
            path("toDir")?,
            flag("cut")
        )?),
        "document_count" => json!(crate::document_count(state)?),
        "open_document" => json!(crate::open_document(path("path")?, state)?),
        "update_document" => json!(crate::update_document(path("path")?, text("text")?, state)?),
        "save_document" => json!(crate::save_document(path("path")?, state)?),
        "close_document" => json!(crate::close_document(path("path")?, state)?),
        "search_workspace" => json!(
            crate::search_workspace(path("root")?, text("query")?, flag("caseSensitive")).await?
        ),
        "find_files" => json!(crate::find_files(path("root")?, text("query")?).await?),
        "git_status" => json!(crate::git::git_status(path("root")?).await?),
        "git_stage" => json!(crate::git::git_stage(path("root")?, list(args, "paths")).await?),
        "git_unstage" => json!(crate::git::git_unstage(path("root")?, list(args, "paths")).await?),
        "git_commit" => json!(crate::git::git_commit(path("root")?, text("message")?).await?),
        "git_push" => json!(crate::git::git_push(path("root")?).await?),
        "git_pull" => json!(crate::git::git_pull(path("root")?).await?),
        "terminal_change_dir" => {
            let dir = crate::terminal_change_dir(path("currentDir")?, text("target")?)?;
            if !inside(&remote.root, Path::new(&dir)) {
                return Err("cd: that folder is outside the opened folder".to_string());
            }
            json!(dir)
        }
        "terminal_poll" => json!(std::mem::take(&mut *remote.terminal.lock().unwrap())),

        "execute_code" | "stop_code" | "execute_terminal_command" | "stop_terminal_command"
            if !remote.allow_run =>
        {
            return Err(
                "Running code from the phone is off. Turn it on in Settings on the laptop"
                    .to_string(),
            )
        }
        "execute_code" => {
            json!(crate::execute_code(path("filePath")?, text("language")?, text("code")?).await)
        }
        "stop_code" => json!(crate::stop_code()?),
        "execute_terminal_command" => {
            start_terminal(remote, text("command")?, path("currentDir")?);
            Value::Null
        }
        "stop_terminal_command" => json!(crate::stop_terminal_command()?),
        _ => return Err(format!("\"{}\" does not work from a phone yet", command)),
    };
    Ok(result)
}

fn start_terminal(remote: &Remote, command: String, dir: String) {
    let events = remote.terminal.clone();
    events.lock().unwrap().clear();

    std::thread::spawn(move || {
        let sink = events.clone();
        let result = crate::run_terminal_command(&command, &dir, move |event| {
            let mut events = sink.lock().unwrap();
            let done = matches!(event, crate::TerminalEvent::Done { .. });

            if done || events.len() < MAX_TERMINAL_LINES {
                events.push(event);
            }
        });

        if let Err(error) = result {
            let mut events = events.lock().unwrap();
            events.push(crate::TerminalEvent::Line {
                text: error,
                error: true,
            });
            events.push(crate::TerminalEvent::Done { code: -1 });
        }
    });
}

fn list(args: &Value, name: &str) -> Vec<String> {
    args.get(name)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

// true if the path is the opened folder or inside it. new files are checked by their folder
fn inside(root: &Path, path: &Path) -> bool {
    let mut existing = path;
    while !existing.exists() {
        match existing.parent() {
            Some(parent) => existing = parent,
            None => return false,
        }
    }

    let new_part = path.strip_prefix(existing).unwrap_or(path);
    if new_part.components().any(|c| c == Component::ParentDir) {
        return false;
    }

    existing
        .canonicalize()
        .map(|real| real.starts_with(root))
        .unwrap_or(false)
}
