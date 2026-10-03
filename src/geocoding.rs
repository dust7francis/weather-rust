use crate::{fetch_7day_weather, format_day_of_week, DailyWeather};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const DEFAULT_GEOCODING_BASE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
pub const DEFAULT_WEATHER_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeocodingLocation {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub elevation: Option<f64>,
    #[serde(default)]
    pub feature_code: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub admin1: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub population: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeocodingResponse {
    #[serde(default)]
    pub results: Option<Vec<GeocodingLocation>>,
    #[serde(default)]
    pub generationtime_ms: Option<f64>,
}

pub fn lookup_location(
    client: &Client,
    geocoding_base_url: &str,
    name: &str,
) -> Result<Option<GeocodingLocation>, Box<dyn Error>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Location name cannot be empty".into());
    }

    let response = client
        .get(geocoding_base_url)
        .query(&[("name", trimmed), ("count", "1"), ("language", "en")])
        .send()?
        .error_for_status()?
        .json::<GeocodingResponse>()?;

    Ok(response.results.and_then(|mut r| {
        if r.is_empty() {
            None
        } else {
            Some(r.remove(0))
        }
    }))
}

pub fn get_7day_weather_for_coords(
    client: &Client,
    weather_base_url: &str,
    lat: f64,
    lon: f64,
    timezone: &str,
) -> Result<DailyWeather, Box<dyn Error>> {
    let url = format!(
        "{weather_base_url}?latitude={lat}&longitude={lon}&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone={timezone}&forecast_days=7"
    );
    fetch_7day_weather(client, &url)
}

pub fn format_7day_forecast(location: &GeocodingLocation, weather: &DailyWeather) -> String {
    let mut out = String::new();
    let display_name = match (&location.admin1, &location.country) {
        (Some(admin), Some(country)) if !admin.is_empty() && !country.is_empty() => {
            format!("{}, {}, {}", location.name, admin, country)
        }
        (_, Some(country)) if !country.is_empty() => {
            format!("{}, {}", location.name, country)
        }
        _ => location.name.clone(),
    };

    let tz = location.timezone.as_deref().unwrap_or("auto");
    out.push_str(&format!("☀️ {} 7-Day Weather Forecast ☀️\n", display_name));
    out.push_str(&format!(
        "Coordinates: {:.4}, {:.4} | Timezone: {}\n\n",
        location.latitude, location.longitude, tz
    ));
    out.push_str(&format!(
        "{:<12} {:<12} {:>8} {:>8} {:>8}\n",
        "Date", "Day", "Min", "Max", "Rain"
    ));

    for i in 0..weather.time.len() {
        let date = &weather.time[i];
        let day = format_day_of_week(date);
        let min_temp = weather.temperature_2m_min.get(i).copied().unwrap_or(0.0);
        let max_temp = weather.temperature_2m_max.get(i).copied().unwrap_or(0.0);
        let rain = weather
            .precipitation_probability_max
            .get(i)
            .copied()
            .unwrap_or(0);

        out.push_str(&format!(
            "{:<12} {:<12} {:>6.1}°C {:>6.1}°C {:>7}%\n",
            date, day, min_temp, max_temp, rain
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_server(status_line: &str, body: &str) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
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
    fn test_deserialize_geocoding_response_success() {
        let payload = r#"{
            "results": [{
                "id": 1850147,
                "name": "Tokyo",
                "latitude": 35.6895,
                "longitude": 139.69171,
                "elevation": 44.0,
                "country_code": "JP",
                "timezone": "Asia/Tokyo",
                "country": "Japan",
                "admin1": "Tokyo"
            }]
        }"#;

        let response: GeocodingResponse =
            serde_json::from_str(payload).expect("Failed to parse GeocodingResponse");
        let results = response.results.expect("results should not be None");
        assert_eq!(results.len(), 1);
        let loc = &results[0];
        assert_eq!(loc.name, "Tokyo");
        assert_eq!(loc.country.as_deref(), Some("Japan"));
        assert_eq!(loc.timezone.as_deref(), Some("Asia/Tokyo"));
        assert!((loc.latitude - 35.6895).abs() < f64::EPSILON);
        assert!((loc.longitude - 139.69171).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_geocoding_not_found() {
        let payload = r#"{"generationtime_ms": 0.5}"#;
        let response: GeocodingResponse =
            serde_json::from_str(payload).expect("Failed to parse empty GeocodingResponse");
        assert!(response.results.is_none());
    }

    #[test]
    fn test_lookup_location_empty_name() {
        let client = Client::new();
        let result = lookup_location(&client, DEFAULT_GEOCODING_BASE_URL, "   ");
        assert!(result.is_err(), "Empty name should return an error");
    }

    #[test]
    fn test_lookup_location_mock_success() {
        let mock_body = r#"{
            "results": [{
                "id": 2147714,
                "name": "Sydney",
                "latitude": -33.8678,
                "longitude": 151.2073,
                "country": "Australia",
                "timezone": "Australia/Sydney"
            }]
        }"#;
        let (url, handle) = spawn_mock_server("HTTP/1.1 200 OK", mock_body);
        let client = Client::new();
        let result = lookup_location(&client, &url, "Sydney");
        let _ = handle.join();

        let loc = result
            .expect("Lookup should succeed")
            .expect("Location found");
        assert_eq!(loc.name, "Sydney");
        assert_eq!(loc.country.as_deref(), Some("Australia"));
    }

    #[test]
    fn test_lookup_location_mock_not_found() {
        let mock_body = r#"{"generationtime_ms": 0.4}"#;
        let (url, handle) = spawn_mock_server("HTTP/1.1 200 OK", mock_body);
        let client = Client::new();
        let result = lookup_location(&client, &url, "UnknownPlace999");
        let _ = handle.join();

        let loc = result.expect("Lookup should succeed with None");
        assert!(loc.is_none());
    }

    #[test]
    fn test_format_7day_forecast_output() {
        let location = GeocodingLocation {
            id: Some(1),
            name: "Paris".to_string(),
            latitude: 48.8534,
            longitude: 2.3488,
            elevation: Some(35.0),
            feature_code: None,
            country_code: Some("FR".to_string()),
            country: Some("France".to_string()),
            admin1: Some("Ile-de-France".to_string()),
            timezone: Some("Europe/Paris".to_string()),
            population: None,
        };

        let weather = DailyWeather {
            time: vec!["2026-10-03".to_string(), "2026-10-04".to_string()],
            temperature_2m_max: vec![18.0, 19.5],
            temperature_2m_min: vec![10.0, 11.2],
            precipitation_probability_max: vec![5, 40],
        };

        let formatted = format_7day_forecast(&location, &weather);
        assert!(formatted.contains("☀️ Paris, Ile-de-France, France 7-Day Weather Forecast ☀️"));
        assert!(formatted.contains("Coordinates: 48.8534, 2.3488 | Timezone: Europe/Paris"));
        assert!(formatted.contains("Date") && formatted.contains("Rain"));
        assert!(formatted.contains("2026-10-03") && formatted.contains("Saturday"));
        assert!(formatted.contains("2026-10-04") && formatted.contains("Sunday"));
    }
}
