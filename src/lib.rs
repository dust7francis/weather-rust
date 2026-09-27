use chrono::{DateTime, Local};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

const LAT: f64 = -35.2835;
const LON: f64 = 149.1281;
const TIMEZONE: &str = "Australia/Sydney";

#[derive(Debug, Deserialize)]
pub struct CurrentResponse {
    pub current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
pub struct CurrentWeather {
    pub time: String,
    pub temperature_2m: f64,
    pub wind_speed_10m: f64,
}

#[derive(Debug, Deserialize)]
pub struct DailyResponse {
    pub daily: DailyWeather,
}

#[derive(Debug, Deserialize)]
pub struct DailyWeather {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_probability_max: Vec<u8>,
}

pub fn get_current_weather() -> Result<CurrentWeather, Box<dyn Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={LAT}&longitude={LON}&current=temperature_2m,wind_speed_10m&timezone={TIMEZONE}"
    );

    let client = Client::new();
    let weather = client
        .get(url)
        .send()?
        .error_for_status()?
        .json::<CurrentResponse>()?;

    Ok(weather.current)
}

pub fn get_7day_weather() -> Result<DailyWeather, Box<dyn Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={LAT}&longitude={LON}&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone={TIMEZONE}&forecast_days=7"
    );

    let client = Client::new();
    let weather = client
        .get(url)
        .send()?
        .error_for_status()?
        .json::<DailyResponse>()?;

    Ok(weather.daily)
}

pub fn current_local_time() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}
