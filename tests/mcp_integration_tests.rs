use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use weather_rust::McpServer;

#[test]
fn test_mcp_server_initialize() {
    let server = McpServer::new();
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let resp = server.handle_line(req).expect("Response expected");
    let json: serde_json::Value = serde_json::from_str(&resp).expect("Valid JSON");

    assert_eq!(json["id"], 1);
    assert_eq!(
        json["result"]["serverInfo"]["name"],
        "Global Weather & Location Server"
    );
    assert_eq!(json["result"]["serverInfo"]["version"], "0.1.0");
    assert_eq!(json["result"]["protocolVersion"], "2024-11-05");
}

#[test]
fn test_mcp_server_tools_list() {
    let server = McpServer::new();
    let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let resp = server.handle_line(req).expect("Response expected");
    let json: serde_json::Value = serde_json::from_str(&resp).expect("Valid JSON");

    assert_eq!(json["id"], 2);
    let tools = json["result"]["tools"].as_array().expect("Tools array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(tool_names.contains(&"get_7day_weather"));
    assert!(tool_names.contains(&"lookup_location"));
}

#[test]
fn test_mcp_binary_stdio_process() {
    let bin_path = env!("CARGO_BIN_EXE_global_weather_mcp");
    let mut child = Command::new(bin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn MCP binary");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Send initialize request
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":100,"method":"initialize","params":{{}}}}"#
    )
    .expect("Failed to write to stdin");

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read line from stdout");

    let json: serde_json::Value =
        serde_json::from_str(&response_line).expect("Valid JSON from MCP binary stdout");
    assert_eq!(json["id"], 100);
    assert_eq!(
        json["result"]["serverInfo"]["name"],
        "Global Weather & Location Server"
    );

    // Send tools/list request
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":101,"method":"tools/list","params":{{}}}}"#
    )
    .expect("Failed to write to stdin");

    response_line.clear();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read tools/list response");

    let tools_json: serde_json::Value =
        serde_json::from_str(&response_line).expect("Valid JSON from tools/list");
    assert_eq!(tools_json["id"], 101);

    drop(stdin);
    let status = child.wait().expect("Failed to wait on child");
    assert!(status.success());
}

/// Live integration test against Open-Meteo Geocoding and Forecast APIs.
/// Run with: cargo test --test mcp_integration_tests -- --ignored
#[test]
#[ignore]
fn test_live_mcp_tool_get_7day_weather() {
    let server = McpServer::new();
    let req = r#"{
        "jsonrpc": "2.0",
        "id": 200,
        "method": "tools/call",
        "params": {
            "name": "get_7day_weather",
            "arguments": {
                "name": "Canberra"
            }
        }
    }"#;

    let resp = server.handle_line(req).expect("Response expected");
    let json: serde_json::Value = serde_json::from_str(&resp).expect("Valid JSON");

    assert_eq!(json["id"], 200);
    assert_eq!(json["result"]["isError"], false);
    let text = json["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    assert!(text.contains("Canberra"));
    assert!(text.contains("7-Day Weather Forecast"));
    assert!(text.contains("Min"));
    assert!(text.contains("Max"));
    assert!(text.contains("Rain"));
}

/// Live integration test for location lookup tool.
/// Run with: cargo test --test mcp_integration_tests -- --ignored
#[test]
#[ignore]
fn test_live_mcp_tool_lookup_location() {
    let server = McpServer::new();
    let req = r#"{
        "jsonrpc": "2.0",
        "id": 201,
        "method": "tools/call",
        "params": {
            "name": "lookup_location",
            "arguments": {
                "name": "Tokyo"
            }
        }
    }"#;

    let resp = server.handle_line(req).expect("Response expected");
    let json: serde_json::Value = serde_json::from_str(&resp).expect("Valid JSON");

    assert_eq!(json["id"], 201);
    assert_eq!(json["result"]["isError"], false);
    let text = json["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    assert!(text.contains("Location: Tokyo"));
    assert!(text.contains("Japan"));
    assert!(text.contains("Coordinates:"));
}
