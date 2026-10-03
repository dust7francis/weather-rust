use crate::geocoding::{
    format_7day_forecast, get_7day_weather_for_coords, lookup_location, DEFAULT_GEOCODING_BASE_URL,
    DEFAULT_WEATHER_BASE_URL,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const SERVER_NAME: &str = "Global Weather & Location Server";
pub const SERVER_VERSION: &str = "0.1.0";
pub const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct RawJsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolCallContent>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

pub struct McpServer {
    client: Client,
    geocoding_base_url: String,
    weather_base_url: String,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    pub fn new() -> Self {
        let timeout_secs = std::env::var("API_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(10);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        let geocoding_base_url = std::env::var("GEOCODING_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_GEOCODING_BASE_URL.to_string());
        let weather_base_url = std::env::var("WEATHER_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_WEATHER_BASE_URL.to_string());

        Self {
            client,
            geocoding_base_url,
            weather_base_url,
        }
    }

    pub fn with_urls(
        client: Client,
        geocoding_base_url: impl Into<String>,
        weather_base_url: impl Into<String>,
    ) -> Self {
        Self {
            client,
            geocoding_base_url: geocoding_base_url.into(),
            weather_base_url: weather_base_url.into(),
        }
    }

    pub fn handle_line(&self, line: &str) -> Option<String> {
        let request: RawJsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(_) => {
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    }
                });
                return Some(err_res.to_string());
            }
        };

        // If message has no id, it's a notification: do not respond.
        let id = request.id?;

        let response = match request.method.as_str() {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": SERVER_NAME,
                        "version": SERVER_VERSION
                    }
                }
            }),

            "ping" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }),

            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "get_7day_weather",
                            "description": "Get the 7-day weather forecast for any city or place in the world by name (e.g. Canberra, Tokyo, Paris, San Francisco). Looks up coordinates and returns min/max temperatures and rain probabilities.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "description": "The name of the city or place to look up"
                                    }
                                },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "lookup_location",
                            "description": "Look up geographic coordinates, country, region, and timezone for any city or place name in the world.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "description": "The name of the city or place to look up"
                                    }
                                },
                                "required": ["name"]
                            }
                        }
                    ]
                }
            }),

            "tools/call" => {
                let tool_result = self.execute_tool(request.params.as_ref());
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": tool_result
                })
            }

            unknown_method => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {unknown_method}")
                }
            }),
        };

        Some(response.to_string())
    }

    fn execute_tool(&self, params: Option<&serde_json::Value>) -> ToolCallResult {
        let params_obj = match params.and_then(|p| p.as_object()) {
            Some(obj) => obj,
            None => {
                return ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text: "Error: Missing parameters for tools/call".to_string(),
                    }],
                    is_error: true,
                }
            }
        };

        let tool_name = match params_obj.get("name").and_then(|n| n.as_str()) {
            Some(name) => name,
            None => {
                return ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text: "Error: Missing 'name' in tools/call parameters".to_string(),
                    }],
                    is_error: true,
                }
            }
        };

        let arguments = params_obj.get("arguments").and_then(|a| a.as_object());
        let place_name = arguments
            .and_then(|a| a.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if place_name.is_empty() {
            return ToolCallResult {
                content: vec![ToolCallContent {
                    content_type: "text".to_string(),
                    text: "Error: 'name' argument is required and cannot be empty.".to_string(),
                }],
                is_error: true,
            };
        }

        match tool_name {
            "get_7day_weather" | "weather7days" | "weather_7day" | "weather-7day" => {
                self.call_get_7day_weather(place_name)
            }
            "lookup_location" => self.call_lookup_location(place_name),
            _ => ToolCallResult {
                content: vec![ToolCallContent {
                    content_type: "text".to_string(),
                    text: format!("Error: Unknown tool '{tool_name}'"),
                }],
                is_error: true,
            },
        }
    }

    fn call_get_7day_weather(&self, place_name: &str) -> ToolCallResult {
        let location = match lookup_location(&self.client, &self.geocoding_base_url, place_name) {
            Ok(Some(loc)) => loc,
            Ok(None) => {
                return ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Location not found: '{place_name}'. Please verify the spelling."
                        ),
                    }],
                    is_error: true,
                };
            }
            Err(e) => {
                return ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text: format!("Error looking up location '{place_name}': {e}"),
                    }],
                    is_error: true,
                };
            }
        };

        let tz = location.timezone.as_deref().unwrap_or("auto");
        match get_7day_weather_for_coords(
            &self.client,
            &self.weather_base_url,
            location.latitude,
            location.longitude,
            tz,
        ) {
            Ok(daily) => {
                let formatted = format_7day_forecast(&location, &daily);
                ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text: formatted,
                    }],
                    is_error: false,
                }
            }
            Err(e) => ToolCallResult {
                content: vec![ToolCallContent {
                    content_type: "text".to_string(),
                    text: format!("Error fetching weather forecast for '{place_name}': {e}"),
                }],
                is_error: true,
            },
        }
    }

    fn call_lookup_location(&self, place_name: &str) -> ToolCallResult {
        match lookup_location(&self.client, &self.geocoding_base_url, place_name) {
            Ok(Some(loc)) => {
                let mut text = format!("📍 Location: {}\n", loc.name);
                if let Some(country) = &loc.country {
                    text.push_str(&format!("Country: {}\n", country));
                }
                if let Some(admin) = &loc.admin1 {
                    text.push_str(&format!("Region: {}\n", admin));
                }
                text.push_str(&format!(
                    "Coordinates: {:.4}, {:.4}\n",
                    loc.latitude, loc.longitude
                ));
                if let Some(tz) = &loc.timezone {
                    text.push_str(&format!("Timezone: {}\n", tz));
                }
                if let Some(elev) = loc.elevation {
                    text.push_str(&format!("Elevation: {:.0}m\n", elev));
                }
                if let Some(pop) = loc.population {
                    text.push_str(&format!("Population: {}\n", pop));
                }

                ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".to_string(),
                        text,
                    }],
                    is_error: false,
                }
            }
            Ok(None) => ToolCallResult {
                content: vec![ToolCallContent {
                    content_type: "text".to_string(),
                    text: format!("Location not found: '{place_name}'."),
                }],
                is_error: true,
            },
            Err(e) => ToolCallResult {
                content: vec![ToolCallContent {
                    content_type: "text".to_string(),
                    text: format!("Error looking up location '{place_name}': {e}"),
                }],
                is_error: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_server(
        responses: Vec<(String, String)>,
    ) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");

        let handle = std::thread::spawn(move || {
            for (status, body) in responses {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 2048];
                    let _ = stream.read(&mut buf);
                    let resp = format!(
                        "{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                }
            }
        });
        (url, handle)
    }

    #[test]
    fn test_initialize_response() {
        let server = McpServer::new();
        let line = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value =
            serde_json::from_str(&response_str).expect("Valid JSON-RPC response");

        assert_eq!(response["id"], 1);
        assert_eq!(
            response["result"]["serverInfo"]["name"],
            "Global Weather & Location Server"
        );
        assert_eq!(response["result"]["serverInfo"]["version"], "0.1.0");
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn test_notification_no_response() {
        let server = McpServer::new();
        let line = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let response = server.handle_line(line);
        assert!(
            response.is_none(),
            "Notifications must not produce a response"
        );
    }

    #[test]
    fn test_ping_response() {
        let server = McpServer::new();
        let line = r#"{"jsonrpc":"2.0","id":42,"method":"ping"}"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(response["id"], 42);
        assert_eq!(response["result"], json!({}));
    }

    #[test]
    fn test_tools_list_response() {
        let server = McpServer::new();
        let line = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();

        assert_eq!(response["id"], 2);
        let tools = response["result"]["tools"].as_array().expect("Tools array");
        assert!(tools.iter().any(|t| t["name"] == "get_7day_weather"));
        assert!(tools.iter().any(|t| t["name"] == "lookup_location"));
    }

    #[test]
    fn test_tools_call_missing_name_argument() {
        let server = McpServer::new();
        let line = r#"{
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_7day_weather",
                "arguments": {}
            }
        }"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();

        assert_eq!(response["id"], 3);
        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("required"));
    }

    #[test]
    fn test_tools_call_unknown_tool() {
        let server = McpServer::new();
        let line = r#"{
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "non_existent_tool",
                "arguments": {"name": "Paris"}
            }
        }"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();

        assert_eq!(response["id"], 4);
        assert_eq!(response["result"]["isError"], true);
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Unknown tool"));
    }

    #[test]
    fn test_tools_call_get_7day_weather_mock_flow() {
        let geocode_json = r#"{
            "results": [{
                "id": 1,
                "name": "Canberra",
                "latitude": -35.2835,
                "longitude": 149.1281,
                "country": "Australia",
                "timezone": "Australia/Sydney"
            }]
        }"#;

        let forecast_json = r#"{
            "daily": {
                "time": ["2026-10-03"],
                "temperature_2m_max": [20.5],
                "temperature_2m_min": [9.0],
                "precipitation_probability_max": [10]
            }
        }"#;

        let (mock_url, handle) = spawn_mock_server(vec![
            ("HTTP/1.1 200 OK".to_string(), geocode_json.to_string()),
            ("HTTP/1.1 200 OK".to_string(), forecast_json.to_string()),
        ]);

        let client = Client::new();
        let server = McpServer::with_urls(client, &mock_url, &mock_url);

        let line = r#"{
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "get_7day_weather",
                "arguments": {"name": "Canberra"}
            }
        }"#;

        let response_str = server.handle_line(line).expect("Expected response");
        let _ = handle.join();

        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(response["id"], 5);
        assert_eq!(response["result"]["isError"], false);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Canberra, Australia 7-Day Weather Forecast"));
        assert!(text.contains("2026-10-03"));
        assert!(text.contains("Saturday"));
        assert!(text.contains("9.0°C"));
        assert!(text.contains("20.5°C"));
    }

    #[test]
    fn test_tools_call_lookup_location_mock() {
        let geocode_json = r#"{
            "results": [{
                "id": 1,
                "name": "Canberra",
                "latitude": -35.2835,
                "longitude": 149.1281,
                "country": "Australia",
                "admin1": "Australian Capital Territory",
                "timezone": "Australia/Sydney",
                "elevation": 580.0
            }]
        }"#;

        let (mock_url, handle) = spawn_mock_server(vec![(
            "HTTP/1.1 200 OK".to_string(),
            geocode_json.to_string(),
        )]);

        let client = Client::new();
        let server = McpServer::with_urls(client, &mock_url, &mock_url);

        let line = r#"{
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "lookup_location",
                "arguments": {"name": "Canberra"}
            }
        }"#;

        let response_str = server.handle_line(line).expect("Expected response");
        let _ = handle.join();

        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(response["id"], 6);
        assert_eq!(response["result"]["isError"], false);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Location: Canberra"));
        assert!(text.contains("Country: Australia"));
        assert!(text.contains("Region: Australian Capital Territory"));
        assert!(text.contains("Coordinates: -35.2835, 149.1281"));
        assert!(text.contains("Timezone: Australia/Sydney"));
    }

    #[test]
    fn test_parse_error() {
        let server = McpServer::new();
        let line = "invalid { json";
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(response["error"]["code"], -32700);
    }

    #[test]
    fn test_unknown_method() {
        let server = McpServer::new();
        let line = r#"{"jsonrpc":"2.0","id":99,"method":"unknown_method"}"#;
        let response_str = server.handle_line(line).expect("Expected response");
        let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(response["id"], 99);
        assert_eq!(response["error"]["code"], -32601);
    }
}
