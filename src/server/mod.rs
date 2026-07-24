#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::db::Database;
use crate::error::Result;

/// Simple HTTP server for GUI
pub struct GuiServer {
    port: u16,
    data_dir: String,
}

impl GuiServer {
    pub fn new(port: u16, data_dir: &str) -> Self {
        Self {
            port,
            data_dir: data_dir.to_string(),
        }
    }
    
    pub fn start(&self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)?;
        
        println!("🚀 Tywindb GUI Server running at http://{}", addr);
        println!("   Press Ctrl+C to stop");
        
        let db = Arc::new(Mutex::new(Database::open(&self.data_dir)?));
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = db.clone();
                    let gui_dir = self.gui_dir();
                    std::thread::spawn(move || {
                        if let Err(e) = handle_client(stream, db, &gui_dir) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    fn gui_dir(&self) -> String {
        // Look for GUI files in multiple locations
        let paths = [
            "gui".to_string(),
            "../gui".to_string(),
            format!("{}/gui", self.data_dir),
        ];
        
        for path in &paths {
            if Path::new(path).exists() {
                return path.clone();
            }
        }
        
        "gui".to_string()
    }
}

fn handle_client(stream: TcpStream, db: Arc<Mutex<Database>>, gui_dir: &str) -> Result<()> {
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let mut writer = stream;
    
    // Read request line
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    
    let method = parts[0];
    let path = parts[1];
    
    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        
        if line.trim().is_empty() {
            break;
        }
        
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    
    // Read body if present
    let mut body = String::new();
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(len) = content_length.parse::<usize>() {
            let mut buffer = vec![0u8; len];
            reader.read_exact(&mut buffer)?;
            body = String::from_utf8(buffer).unwrap_or_default();
        }
    }
    
    // Route request
    let response = match (method, path) {
        ("GET", "/") => serve_file(gui_dir, "index.html"),
        ("GET", "/style.css") => serve_file(gui_dir, "style.css"),
        ("GET", "/app.js") => serve_file(gui_dir, "app.js"),
        ("GET", "/logo.svg") => serve_file(gui_dir, "logo.svg"),
        ("POST", "/api/query") => handle_query(db, &body),
        ("GET", "/api/tables") => handle_list_tables(db),
        _ => {
            let body = r#"{"error": "Not found"}"#;
            format!("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
        }
    };
    
    writer.write_all(response.as_bytes())?;
    
    Ok(())
}

fn serve_file(dir: &str, filename: &str) -> String {
    let path = format!("{}/{}", dir, filename);
    
    let content_type = match filename {
        "index.html" => "text/html",
        "style.css" => "text/css",
        "app.js" => "application/javascript",
        "logo.svg" => "image/svg+xml",
        _ => "text/plain",
    };
    
    match fs::read_to_string(&path) {
        Ok(content) => {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                content_type,
                content.len(),
                content
            )
        }
        Err(_) => {
            let body = "File not found";
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
        }
    }
}

fn handle_query(db: Arc<Mutex<Database>>, body: &str) -> String {
    // Parse JSON body
    let request: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            let response = serde_json::json!({"error": format!("Invalid JSON: {}", e)});
            return format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response.to_string().len(),
                response
            );
        }
    };
    
    let query = match request["query"].as_str() {
        Some(q) => q,
        None => {
            let response = serde_json::json!({"error": "Missing query field"});
            return format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response.to_string().len(),
                response
            );
        }
    };
    
    // Execute query
    let mut db = db.lock().unwrap();
    
    match db.query(query) {
        Ok(result) => {
            let response = match result {
                crate::engine::QueryResult::Rows(rows) => {
                    serde_json::json!({"data": rows})
                }
                crate::engine::QueryResult::RowsAffected(n) => {
                    serde_json::json!({"rows_affected": n})
                }
                crate::engine::QueryResult::TableCreated => {
                    serde_json::json!({"message": "Table created"})
                }
                _ => {
                    serde_json::json!({"message": "Query executed"})
                }
            };
            
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                response.to_string().len(),
                response
            )
        }
        Err(e) => {
            let response = serde_json::json!({"error": e.to_string()});
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                response.to_string().len(),
                response
            )
        }
    }
}

fn handle_list_tables(db: Arc<Mutex<Database>>) -> String {
    let mut db = db.lock().unwrap();
    
    // Query for table names using SQL
    // For now, return a simple list - in a real implementation, we'd query the schema
    let tables = match db.query("SELECT name FROM sqlite_master WHERE type='table'") {
        Ok(crate::engine::QueryResult::Rows(rows)) => {
            rows.iter()
                .filter_map(|row| row.get("name").and_then(|v| v.as_str()).map(String::from))
                .collect::<Vec<_>>()
        }
        _ => {
            // Fallback: try to get tables from executor
            Vec::new()
        }
    };
    
    let response = serde_json::json!({"tables": tables});
    
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        response.to_string().len(),
        response
    )
}
