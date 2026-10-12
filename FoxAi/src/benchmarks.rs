pub fn get_prompts() -> [(&'static str, &'static str, &'static str); 30] {
    [
        (
            "Vector 2D Math (dot product)",
            r#"use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: &Vec2) -> f32 {
        "#,
            r#"
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            *self
        }
    }
}"#,
        ),
        (
            "File JSON Configuration Loader",
            r#"use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            debug: false,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        "#,
            r#"
    }
}"#,
        ),
        (
            "HTTP Response Status Check",
            r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok,
    Created,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            200 => Some(HttpStatus::Ok),
            201 => Some(HttpStatus::Created),
            400 => Some(HttpStatus::BadRequest),
            404 => Some(HttpStatus::NotFound),
            500 => Some(HttpStatus::InternalServerError),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        "#,
            r#"
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "200 OK",
            HttpStatus::Created => "201 Created",
            HttpStatus::BadRequest => "400 Bad Request",
            HttpStatus::NotFound => "404 Not Found",
            HttpStatus::InternalServerError => "500 Internal Server Error",
        }
    }
}"#,
        ),
        (
            "Thread-Safe Cache Get",
            r#"use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CacheEntry<V> {
    pub value: V,
    pub hits: usize,
}

#[derive(Clone)]
pub struct ThreadSafeCache<K: std::hash::Hash + Eq, V: Clone> {
    entries: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
}

impl<K: std::hash::Hash + Eq, V: Clone> ThreadSafeCache<K, V> {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut map = self.entries.lock().ok()?;
        "#,
            r#"
    }
}"#,
        ),
        (
            "Binary Search Tree Insertion",
            r#"#[derive(Debug)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl TreeNode {
    pub fn new(val: i32) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }

    pub fn insert(&mut self, new_val: i32) {
        if new_val < self.val {
            match self.left {
                Some(ref mut left_node) => left_node.insert(new_val),
                None => self.left = Some(Box::new(TreeNode::new(new_val))),
            }
        } else if new_val > self.val {
            "#,
            r#"
        }
    }
}"#,
        ),
        (
            "Custom Error Display Implementation",
            r#"use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    ConnectionFailed(String),
    RecordNotFound(u64),
    QueryTimeout,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(msg) => write!(f, "Connection error: {}", msg),
            DatabaseError::RecordNotFound(id) => write!(f, "Record with ID {} was not found", id),
            DatabaseError::QueryTimeout => "#,
            r#"
        }
    }
}

impl std::error::Error for DatabaseError {}"#,
        ),
        (
            "Lexer Skip Whitespace",
            r#"#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(i64),
    Plus,
    Minus,
    Eof,
}

pub struct Lexer<'a> {
    src: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, cursor: 0 }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.src[self.cursor..].chars().next()
    }

    pub fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                "#,
            r#"
            }
        }
    }
}"#,
        ),
        (
            "Server Builder Build Method",
            r#"use std::net::SocketAddr;

pub struct Server {
    addr: SocketAddr,
    max_connections: usize,
    tls_enabled: bool,
}

pub struct ServerBuilder {
    addr: Option<SocketAddr>,
    max_connections: usize,
    tls_enabled: bool,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            addr: None,
            max_connections: 1024,
            tls_enabled: false,
        }
    }

    pub fn with_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    pub fn with_tls(mut self, enabled: bool) -> Self {
        self.tls_enabled = enabled;
        self
    }

    pub fn build(self) -> Result<Server, &'static str> {
        let addr = self.addr.ok_or("Server address must be specified")?;
        Ok(Server {
            addr,
            max_connections: "#,
            r#",
            tls_enabled: self.tls_enabled,
        })
    }
}"#,
        ),
        (
            "Word Counter Most Frequent",
            r#"use std::collections::HashMap;

pub struct WordCounter;

impl WordCounter {
    pub fn count_words(text: &str) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for word in text.split_whitespace() {
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            if clean.is_empty() {
                continue;
            }

            *freq.entry(clean).or_insert(0) += 1;
        }
        freq
    }

    pub fn most_frequent(freq: &HashMap<String, usize>) -> Option<(&String, &usize)> {
        "#,
            r#"
    }
}"#,
        ),
        (
            "Shell Command Runner Error Branch",
            r#"use std::process::{Command, Output};
use std::io::{self, Error, ErrorKind};

pub struct Shell;

impl Shell {
    pub fn run_command(program: &str, args: &[&str]) -> io::Result<String> {
        let output: Output = Command::new(program)
            .args(args)
            .output()?;

        if output.status.success() {
            let text = String::from_utf8(output.stdout)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
            Ok(text.trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            "#,
            r#"
        }
    }
}"#,
        ),
        (
            "Matrix Multiplication Dot Inner Loop",
            r#"#[derive(Debug, Clone, PartialEq)]
pub struct Matrix2D {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
}

impl Matrix2D {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn get(&self, r: usize, c: usize) -> f32 {
        self.data[r * self.cols + c]
    }

    pub fn set(&mut self, r: usize, c: usize, val: f32) {
        self.data[r * self.cols + c] = val;
    }

    pub fn multiply(&self, other: &Matrix2D) -> Option<Matrix2D> {
        if self.cols != other.rows {
            return None;
        }

        let mut out = Matrix2D::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    "#,
            r#"
                }
                out.set(i, j, sum);
            }
        }
        Some(out)
    }
}"#,
        ),
        (
            "Bitfield Permission Remove Flag",
            r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions(pub u32);

impl Permissions {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const EXECUTE: u32 = 1 << 2;
    pub const ADMIN: u32 = 1 << 3;

    pub fn empty() -> Self {
        Permissions(0)
    }

    pub fn all() -> Self {
        Permissions(Self::READ | Self::WRITE | Self::EXECUTE | Self::ADMIN)
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }

    pub fn add(&mut self, flag: u32) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u32) {
        "#,
            r#"
    }
}"#,
        ),
        (
            "Env Config Secret Key Fallback",
            r#"use std::env;

pub struct EnvConfig {
    pub database_url: String,
    pub port: u16,
    pub secret_key: String,
}

impl EnvConfig {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/app".to_string());

        let port = match env::var("PORT") {
            Ok(val) => val.parse::<u16>().map_err(|_| "Invalid PORT environment variable".to_string())?,
            Err(_) => 8080,
        };

        let secret_key = "#,
            r#";

        Ok(Self {
            database_url,
            port,
            secret_key,
        })
    }
}"#,
        ),
        (
            "Ring Buffer Size Update",
            r#"pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            buffer: vec![None; capacity],
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    pub fn push(&mut self, item: T) {
        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        if self.is_full() {
            self.head = (self.head + 1) % self.capacity;
        } else {
            "#,
            r#"
        }
    }
}"#,
        ),
        (
            "Hex String Decoding Byte Assembly",
            r#"pub struct Hex;

impl Hex {
    const HEX_CHARS: &'static [u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            out.push(Self::HEX_CHARS[(b >> 4) as usize] as char);
            out.push(Self::HEX_CHARS[(b & 0x0F) as usize] as char);
        }
        out
    }

    pub fn decode(hex_str: &str) -> Result<Vec<u8>, &'static str> {
        if hex_str.len() % 2 != 0 {
            return Err("Hex string must have an even length");
        }

        let mut out = Vec::with_capacity(hex_str.len() / 2);
        let bytes = hex_str.as_bytes();
        for chunk in bytes.chunks(2) {
            let high = Self::from_hex_digit(chunk[0])?;
            let low = Self::from_hex_digit(chunk[1])?;
            "#,
            r#"
        }
        Ok(out)
    }

    fn from_hex_digit(b: u8) -> Result<u8, &'static str> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err("Invalid hex character"),
        }
    }
}"#,
        ),
        (
            "BinaryHeap Next Task Pop",
            r#"use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    pub priority: u32,
    pub name: String,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TaskScheduler {
    heap: BinaryHeap<Task>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self { heap: BinaryHeap::new() }
    }

    pub fn add_task(&mut self, name: &str, priority: u32) {
        self.heap.push(Task {
            name: name.to_string(),
            priority,
        });
    }

    pub fn pop_next(&mut self) -> Option<Task> {
        "#,
            r#"
    }
}"#,
        ),
        (
            "LogWriter File Append Write",
            r#"use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub struct LogWriter {
    file_path: String,
}

impl LogWriter {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn write_line(&self, line: &str) -> io::Result<()> {
        let path = Path::new(&self.file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        "#,
            r#"
    }
}"#,
        ),
        (
            "User Profile Bio Update Branch",
            r#"pub struct UserProfile {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
}

pub struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub bio: Option<Option<String>>,
}

impl UserProfile {
    pub fn apply_update(&mut self, update: UserUpdate) {
        if let Some(new_username) = update.username {
            self.username = new_username;
        }

        if let Some(new_email) = update.email {
            self.email = new_email;
        }

        if let Some(new_bio) = update.bio {
            "#,
            r#"
        }
    }
}"#,
        ),
        (
            "Order Cancellation State Transition",
            r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Paid,
    Shipped,
    Delivered,
    Cancelled,
}

pub struct Order {
    pub id: u64,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            status: OrderStatus::Pending,
        }
    }

    pub fn mark_paid(&mut self) -> Result<(), &'static str> {
        match self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Paid;
                Ok(())
            }
            OrderStatus::Cancelled => Err("Cannot pay a cancelled order"),
            _ => Err("Order is already paid or completed"),
        }
    }

    pub fn cancel(&mut self) -> Result<(), &'static str> {
        "#,
            r#"
    }
}"#,
        ),
        (
            "Exponential Backoff Upper Bound Clamp",
            r#"use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl RetryPolicy {
    pub fn default() -> Self {
        Self {
            max_retries: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }

    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return self.base_delay;
        }

        let factor = 2u64.saturating_pow(attempt as u32);
        let millis = self.base_delay.as_millis() as u64 * factor;
        let calculated = Duration::from_millis(millis);

        "#,
            r#"
    }
}"#,
        ),
        (
            "SessionId Static Validation Helper",
            r#"#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(raw: &str) -> Result<Self, &'static str> {
        if raw.len() != 32 {
            return Err("Session ID must be exactly 32 hex characters");
        }

        if !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Session ID must only contain hexadecimal characters");
        }

        Ok(SessionId(raw.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_valid(raw: &str) -> bool {
        "#,
            r#"
    }
}"#,
        ),
        (
            "MemoryStore File Format Serializer",
            r#"use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Write};

pub struct MemoryStore {
    data: BTreeMap<String, String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        for (k, v) in &self.data {
            "#,
            r#"
        }
        file.flush()?;
        Ok(())
    }
}"#,
        ),
        (
            "Checksum Verification Helper",
            r#"pub struct Checksum;

impl Checksum {
    pub fn compute_crc(bytes: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFF_FFFF;
        for &byte in bytes {
            crc ^= byte as u32;
            for _ in 0..8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ 0xEDB8_8320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    pub fn verify(bytes: &[u8], expected: u32) -> bool {
        "#,
            r#"
    }
}"#,
        ),
        (
            "Page Request Slice Extraction",
            r#"#[derive(Debug, PartialEq, Eq)]
pub struct PageRequest {
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub total_pages: usize,
    pub total_items: usize,
}

impl PageRequest {
    pub fn paginate<T: Clone>(&self, all_items: &[T]) -> PageResult<T> {
        let total_items = all_items.len();
        let total_pages = (total_items + self.page_size - 1) / self.page_size.max(1);

        let start = (self.page * self.page_size).min(total_items);
        let end = (start + self.page_size).min(total_items);

        let items = "#,
            r#";

        PageResult {
            items,
            total_pages,
            total_items,
        }
    }
}"#,
        ),
        (
            "Token Bucket Grant Return",
            r#"use std::time::Instant;

pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate_per_sec: f64,
    last_update: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate_per_sec,
            last_update: Instant::now(),
        }
    }

    pub fn allow(&mut self, count: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        self.tokens = (self.tokens + elapsed * self.refill_rate_per_sec).min(self.capacity);

        if self.tokens >= count {
            self.tokens -= count;
            "#,
            r#"
        } else {
            false
        }
    }
}"#,
        ),
        (
            "URL Path Reassembly",
            r#"pub struct UrlHelper;

impl UrlHelper {
    pub fn sanitize_path(raw: &str) -> String {
        let mut segments = Vec::new();

        for part in raw.split('/') {
            match part {
                "" | "." => continue,
                ".." => {
                    segments.pop();
                }
                normal => {
                    segments.push(normal);
                }
            }
        }

        if segments.is_empty() {
            "/".to_string()
        } else {
            "#,
            r#"
        }
    }
}"#,
        ),
        (
            "Worker Thread Job Execution",
            r#"use std::sync::mpsc::{self, Receiver};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: std::sync::Arc<std::sync::Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    "#,
            r#"
                }
                Message::Terminate => break,
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}"#,
        ),
        (
            "Text Distance Return Last Matrix Element",
            r#"pub struct TextDistance;

impl TextDistance {
    pub fn levenshtein(a: &str, b: &str) -> usize {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        let mut row: Vec<usize> = (0..=b_bytes.len()).collect();

        for (i, &ca) in a_bytes.iter().enumerate() {
            let mut prev_diag = row[0];
            row[0] = i + 1;

            for (j, &cb) in b_bytes.iter().enumerate() {
                let old_row_val = row[j + 1];
                let cost = if ca == cb { 0 } else { 1 };

                row[j + 1] = (row[j] + 1)
                    .min(row[j + 1] + 1)
                    .min(prev_diag + cost);

                prev_diag = old_row_val;
            }
        }

        "#,
            r#"
    }
}"#,
        ),
        (
            "Unit Test Assertion Block",
            r#"pub trait Greeter {
    fn greet(&self, name: &str) -> String;
}

pub struct SimpleGreeter;

impl Greeter for SimpleGreeter {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

pub fn welcome_user<G: Greeter>(greeter: &G, user: &str) -> String {
    greeter.greet(user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_user() {
        let greeter = SimpleGreeter;
        let message = welcome_user(&greeter, "Alice");
        "#,
            r#"
    }
}"#,
        ),
        (
            "JsonValue Array Truthy Check",
            r#"#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            JsonValue::Null => false,
            JsonValue::Bool(b) => *b,
            JsonValue::Number(n) => *n != 0.0,
            JsonValue::String(s) => !s.is_empty(),
            JsonValue::Array(arr) => "#,
            r#"
        }
    }
}"#,
        ),
    ]
}
