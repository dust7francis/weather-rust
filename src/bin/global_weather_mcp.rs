use std::io::{self, BufRead, Write};
use weather_rust::mcp::McpServer;

fn main() {
    let server = McpServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(line_str) => {
                let trimmed = line_str.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(response) = server.handle_line(trimmed) {
                    if let Err(e) = writeln!(stdout, "{}", response) {
                        eprintln!("Failed to write to stdout: {e}");
                        break;
                    }
                    if let Err(e) = stdout.flush() {
                        eprintln!("Failed to flush stdout: {e}");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {e}");
                break;
            }
        }
    }
}
