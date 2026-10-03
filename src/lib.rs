use chrono::{DateTime, Local, NaiveDate};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const LAT: f64 = -35.2835;
pub const LON: f64 = 149.1281;
pub const TIMEZONE: &str = "Australia/Sydney";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentResponse {
    pub current: CurrentWeather,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub time: String,
    pub temperature_2m: f64,
    pub wind_speed_10m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyResponse {
    pub daily: DailyWeather,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyWeather {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_probability_max: Vec<u8>,
}

pub fn current_weather_url() -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={LAT}&longitude={LON}&current=temperature_2m,wind_speed_10m&timezone={TIMEZONE}"
    )
}

pub fn seven_day_weather_url() -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={LAT}&longitude={LON}&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone={TIMEZONE}&forecast_days=7"
    )
}

pub fn fetch_current_weather(client: &Client, url: &str) -> Result<CurrentWeather, Box<dyn Error>> {
    let weather = client
        .get(url)
        .send()?
        .error_for_status()?
        .json::<CurrentResponse>()?;

    Ok(weather.current)
}

pub fn fetch_7day_weather(client: &Client, url: &str) -> Result<DailyWeather, Box<dyn Error>> {
    let weather = client
        .get(url)
        .send()?
        .error_for_status()?
        .json::<DailyResponse>()?;

    Ok(weather.daily)
}

pub fn get_current_weather() -> Result<CurrentWeather, Box<dyn Error>> {
    let client = Client::new();
    fetch_current_weather(&client, &current_weather_url())
}

pub fn get_7day_weather() -> Result<DailyWeather, Box<dyn Error>> {
    let client = Client::new();
    fetch_7day_weather(&client, &seven_day_weather_url())
}

pub fn current_local_time() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

pub fn format_day_of_week(date_str: &str) -> String {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.format("%A").to_string())
        .unwrap_or_else(|_| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_server(status_line: &str, body: &str) -> (String, std::thread::JoinHandle<()>) {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("Failed to bind mock server to ephemeral port");
        let port = listener
            .local_addr()
            .expect("Failed to get socket address")
            .port();
        let url = format!("http://127.0.0.1:{port}");
        let response = format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        (url, handle)
    }

    #[test]
    fn test_constants_validity() {
        const { assert!(LAT >= -90.0 && LAT <= 90.0) };
        const { assert!(LON >= -180.0 && LON <= 180.0) };
        assert_eq!(TIMEZONE, "Australia/Sydney");
    }

    #[test]
    fn test_current_weather_url() {
        let url = current_weather_url();
        assert!(url.starts_with("https://api.open-meteo.com/v1/forecast"));
        assert!(url.contains("latitude=-35.2835"));
        assert!(url.contains("longitude=149.1281"));
        assert!(url.contains("current=temperature_2m,wind_speed_10m"));
        assert!(url.contains("timezone=Australia/Sydney"));
    }

    #[test]
    fn test_seven_day_weather_url() {
        let url = seven_day_weather_url();
        assert!(url.starts_with("https://api.open-meteo.com/v1/forecast"));
        assert!(url.contains("latitude=-35.2835"));
        assert!(url.contains("longitude=149.1281"));
        assert!(url
            .contains("daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max"));
        assert!(url.contains("timezone=Australia/Sydney"));
        assert!(url.contains("forecast_days=7"));
    }

    #[test]
    fn test_current_local_time_format() {
        let time_str = current_local_time();
        assert!(!time_str.is_empty(), "Local time should not be empty");
        // Time format is "%Y-%m-%d %H:%M:%S %Z", e.g. "2026-10-03 19:30:00 AEDT" or "+1000"
        let parts: Vec<&str> = time_str.split_whitespace().collect();
        assert!(parts.len() >= 3, "Expected date, time, and timezone parts");
        assert_eq!(parts[0].len(), 10, "Date should be YYYY-MM-DD");
        assert!(NaiveDate::parse_from_str(parts[0], "%Y-%m-%d").is_ok());
    }

    #[test]
    fn test_format_day_of_week_standard_dates() {
        assert_eq!(format_day_of_week("2026-10-03"), "Saturday");
        assert_eq!(format_day_of_week("2026-10-04"), "Sunday");
        assert_eq!(format_day_of_week("2026-10-05"), "Monday");
        assert_eq!(format_day_of_week("2026-10-06"), "Tuesday");
        assert_eq!(format_day_of_week("2026-10-07"), "Wednesday");
        assert_eq!(format_day_of_week("2026-10-08"), "Thursday");
        assert_eq!(format_day_of_week("2026-10-09"), "Friday");
    }

    #[test]
    fn test_format_day_of_week_leap_year() {
        assert_eq!(format_day_of_week("2024-02-29"), "Thursday");
    }

    #[test]
    fn test_format_day_of_week_invalid() {
        assert_eq!(format_day_of_week("invalid-date"), "?");
        assert_eq!(format_day_of_week("2026-02-30"), "?");
        assert_eq!(format_day_of_week("2026-13-01"), "?");
        assert_eq!(format_day_of_week(""), "?");
    }

    #[test]
    fn test_deserialize_current_response_success() {
        let payload = r#"{
            "latitude": -35.2835,
            "longitude": 149.1281,
            "utc_offset_seconds": 36000,
            "timezone": "Australia/Sydney",
            "timezone_abbreviation": "AEST",
            "elevation": 580.0,
            "current_units": {
                "time": "iso8601",
                "interval": "seconds",
                "temperature_2m": "°C",
                "wind_speed_10m": "km/h"
            },
            "current": {
                "time": "2026-10-03T19:30",
                "interval": 900,
                "temperature_2m": 11.7,
                "wind_speed_10m": 10.3
            }
        }"#;

        let response: CurrentResponse =
            serde_json::from_str(payload).expect("Failed to deserialize CurrentResponse");
        assert_eq!(response.current.time, "2026-10-03T19:30");
        assert!((response.current.temperature_2m - 11.7).abs() < f64::EPSILON);
        assert!((response.current.wind_speed_10m - 10.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_current_weather_edge_cases() {
        let payload = r#"{
            "current": {
                "time": "2026-07-15T06:00",
                "temperature_2m": -4.8,
                "wind_speed_10m": 0.0
            }
        }"#;

        let response: CurrentResponse =
            serde_json::from_str(payload).expect("Failed to deserialize edge case CurrentResponse");
        assert_eq!(response.current.time, "2026-07-15T06:00");
        assert!((response.current.temperature_2m - (-4.8)).abs() < f64::EPSILON);
        assert!((response.current.wind_speed_10m - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_current_response_missing_fields() {
        let missing_temp = r#"{
            "current": {
                "time": "2026-10-03T19:30",
                "wind_speed_10m": 10.3
            }
        }"#;
        assert!(serde_json::from_str::<CurrentResponse>(missing_temp).is_err());

        let missing_wind = r#"{
            "current": {
                "time": "2026-10-03T19:30",
                "temperature_2m": 11.7
            }
        }"#;
        assert!(serde_json::from_str::<CurrentResponse>(missing_wind).is_err());

        let missing_time = r#"{
            "current": {
                "temperature_2m": 11.7,
                "wind_speed_10m": 10.3
            }
        }"#;
        assert!(serde_json::from_str::<CurrentResponse>(missing_time).is_err());

        let missing_current = r#"{ "latitude": -35.2835 }"#;
        assert!(serde_json::from_str::<CurrentResponse>(missing_current).is_err());
    }

    #[test]
    fn test_deserialize_current_response_invalid_types() {
        let string_temp = r#"{
            "current": {
                "time": "2026-10-03T19:30",
                "temperature_2m": "warm",
                "wind_speed_10m": 10.3
            }
        }"#;
        assert!(serde_json::from_str::<CurrentResponse>(string_temp).is_err());
    }

    #[test]
    fn test_deserialize_daily_response_success() {
        let payload = r#"{
            "latitude": -35.2835,
            "longitude": 149.1281,
            "timezone": "Australia/Sydney",
            "daily_units": {
                "time": "iso8601",
                "temperature_2m_max": "°C",
                "temperature_2m_min": "°C",
                "precipitation_probability_max": "%"
            },
            "daily": {
                "time": [
                    "2026-10-03", "2026-10-04", "2026-10-05", "2026-10-06",
                    "2026-10-07", "2026-10-08", "2026-10-09"
                ],
                "temperature_2m_max": [17.2, 17.9, 19.2, 20.0, 16.6, 17.5, 19.9],
                "temperature_2m_min": [10.0, 7.7, 8.6, 10.0, 4.9, 5.1, 7.3],
                "precipitation_probability_max": [100, 27, 20, 9, 24, 0, 3]
            }
        }"#;

        let response: DailyResponse =
            serde_json::from_str(payload).expect("Failed to deserialize DailyResponse");
        assert_eq!(response.daily.time.len(), 7);
        assert_eq!(response.daily.temperature_2m_max.len(), 7);
        assert_eq!(response.daily.temperature_2m_min.len(), 7);
        assert_eq!(response.daily.precipitation_probability_max.len(), 7);

        assert_eq!(response.daily.time[0], "2026-10-03");
        assert!((response.daily.temperature_2m_max[0] - 17.2).abs() < f64::EPSILON);
        assert!((response.daily.temperature_2m_min[0] - 10.0).abs() < f64::EPSILON);
        assert_eq!(response.daily.precipitation_probability_max[0], 100);
        assert_eq!(response.daily.precipitation_probability_max[5], 0);
    }

    #[test]
    fn test_deserialize_daily_weather_boundary_conditions() {
        let payload = r#"{
            "daily": {
                "time": ["2026-07-01"],
                "temperature_2m_max": [-1.5],
                "temperature_2m_min": [-8.2],
                "precipitation_probability_max": [100]
            }
        }"#;

        let response: DailyResponse =
            serde_json::from_str(payload).expect("Failed to deserialize boundary DailyResponse");
        assert_eq!(response.daily.time.len(), 1);
        assert!((response.daily.temperature_2m_min[0] - (-8.2)).abs() < f64::EPSILON);
        assert_eq!(response.daily.precipitation_probability_max[0], 100);

        // Empty daily vectors
        let empty_payload = r#"{
            "daily": {
                "time": [],
                "temperature_2m_max": [],
                "temperature_2m_min": [],
                "precipitation_probability_max": []
            }
        }"#;
        let empty_response: DailyResponse = serde_json::from_str(empty_payload)
            .expect("Failed to deserialize empty daily response");
        assert!(empty_response.daily.time.is_empty());
    }

    #[test]
    fn test_deserialize_daily_response_missing_fields() {
        let missing_precip = r#"{
            "daily": {
                "time": ["2026-10-03"],
                "temperature_2m_max": [17.2],
                "temperature_2m_min": [10.0]
            }
        }"#;
        assert!(serde_json::from_str::<DailyResponse>(missing_precip).is_err());

        let missing_daily = r#"{ "latitude": -35.2835 }"#;
        assert!(serde_json::from_str::<DailyResponse>(missing_daily).is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let current_resp = CurrentResponse {
            current: CurrentWeather {
                time: "2026-10-03T19:30".to_string(),
                temperature_2m: 15.5,
                wind_speed_10m: 12.0,
            },
        };
        let serialized =
            serde_json::to_string(&current_resp).expect("Failed to serialize CurrentResponse");
        let deserialized: CurrentResponse =
            serde_json::from_str(&serialized).expect("Failed to deserialize CurrentResponse");
        assert_eq!(current_resp, deserialized);

        let daily_resp = DailyResponse {
            daily: DailyWeather {
                time: vec!["2026-10-03".to_string()],
                temperature_2m_max: vec![20.5],
                temperature_2m_min: vec![10.2],
                precipitation_probability_max: vec![45],
            },
        };
        let serialized_daily =
            serde_json::to_string(&daily_resp).expect("Failed to serialize DailyResponse");
        let deserialized_daily: DailyResponse =
            serde_json::from_str(&serialized_daily).expect("Failed to deserialize DailyResponse");
        assert_eq!(daily_resp, deserialized_daily);
    }

    #[test]
    fn test_fetch_current_weather_mock_success() {
        let mock_body = r#"{
            "current": {
                "time": "2026-10-03T12:00",
                "temperature_2m": 18.5,
                "wind_speed_10m": 14.2
            }
        }"#;
        let (url, handle) = spawn_mock_server("HTTP/1.1 200 OK", mock_body);
        let client = Client::new();
        let result = fetch_current_weather(&client, &url);
        let _ = handle.join();

        let weather = result.expect("fetch_current_weather should succeed on 200 OK");
        assert_eq!(weather.time, "2026-10-03T12:00");
        assert!((weather.temperature_2m - 18.5).abs() < f64::EPSILON);
        assert!((weather.wind_speed_10m - 14.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fetch_current_weather_mock_404_error() {
        let (url, handle) = spawn_mock_server("HTTP/1.1 404 Not Found", "Not Found");
        let client = Client::new();
        let result = fetch_current_weather(&client, &url);
        let _ = handle.join();

        assert!(
            result.is_err(),
            "fetch_current_weather should error on 404 status"
        );
    }

    #[test]
    fn test_fetch_current_weather_mock_500_error() {
        let (url, handle) =
            spawn_mock_server("HTTP/1.1 500 Internal Server Error", "Internal error");
        let client = Client::new();
        let result = fetch_current_weather(&client, &url);
        let _ = handle.join();

        assert!(
            result.is_err(),
            "fetch_current_weather should error on 500 status"
        );
    }

    #[test]
    fn test_fetch_current_weather_mock_malformed_json() {
        let (url, handle) = spawn_mock_server("HTTP/1.1 200 OK", "{ invalid_json: ");
        let client = Client::new();
        let result = fetch_current_weather(&client, &url);
        let _ = handle.join();

        assert!(
            result.is_err(),
            "fetch_current_weather should error on malformed JSON"
        );
    }

    #[test]
    fn test_fetch_7day_weather_mock_success() {
        let mock_body = r#"{
            "daily": {
                "time": ["2026-10-03", "2026-10-04"],
                "temperature_2m_max": [19.0, 21.0],
                "temperature_2m_min": [9.0, 11.0],
                "precipitation_probability_max": [5, 40]
            }
        }"#;
        let (url, handle) = spawn_mock_server("HTTP/1.1 200 OK", mock_body);
        let client = Client::new();
        let result = fetch_7day_weather(&client, &url);
        let _ = handle.join();

        let daily = result.expect("fetch_7day_weather should succeed on 200 OK");
        assert_eq!(daily.time.len(), 2);
        assert_eq!(daily.precipitation_probability_max, vec![5, 40]);
    }

    #[test]
    fn test_fetch_7day_weather_mock_error() {
        let (url, handle) =
            spawn_mock_server("HTTP/1.1 503 Service Unavailable", "Service Unavailable");
        let client = Client::new();
        let result = fetch_7day_weather(&client, &url);
        let _ = handle.join();

        assert!(
            result.is_err(),
            "fetch_7day_weather should error on 503 status"
        );
    }

    #[test]
    fn test_fetch_connection_refused() {
        // Bind and immediately close to obtain an unused port that will refuse connections
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let url = format!("http://127.0.0.1:{port}");
        let client = Client::new();
        let result = fetch_current_weather(&client, &url);
        assert!(
            result.is_err(),
            "fetch should error when connection is refused"
        );
    }
}
