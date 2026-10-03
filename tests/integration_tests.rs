use weather_rust::{
    current_local_time, current_weather_url, format_day_of_week, get_7day_weather,
    get_current_weather, seven_day_weather_url, LAT, LON, TIMEZONE,
};

#[test]
fn test_public_api_constants() {
    assert_eq!(LAT, -35.2835);
    assert_eq!(LON, 149.1281);
    assert_eq!(TIMEZONE, "Australia/Sydney");
}

#[test]
fn test_public_api_urls() {
    let current_url = current_weather_url();
    assert!(current_url.contains("latitude=-35.2835"));
    assert!(current_url.contains("longitude=149.1281"));
    assert!(current_url.contains("current=temperature_2m,wind_speed_10m"));
    assert!(current_url.contains("timezone=Australia/Sydney"));

    let forecast_url = seven_day_weather_url();
    assert!(forecast_url.contains("latitude=-35.2835"));
    assert!(forecast_url.contains("longitude=149.1281"));
    assert!(forecast_url.contains("forecast_days=7"));
    assert!(forecast_url.contains("timezone=Australia/Sydney"));
}

#[test]
fn test_public_api_day_formatting() {
    // Tests known calendar days
    assert_eq!(format_day_of_week("2026-01-01"), "Thursday");
    assert_eq!(format_day_of_week("2026-12-25"), "Friday");
    // Invalid formats
    assert_eq!(format_day_of_week("not-a-date"), "?");
}

#[test]
fn test_public_api_local_time() {
    let local = current_local_time();
    assert!(!local.is_empty());
    assert!(local.starts_with("20"));
}

/// Live integration test against the Open-Meteo API.
/// Marked #[ignore] so default tests run reliably offline.
/// Run explicitly with: cargo test -- --ignored
#[test]
#[ignore]
fn test_live_current_weather() {
    let result = get_current_weather();
    assert!(
        result.is_ok(),
        "Failed to fetch live current weather: {:?}",
        result.err()
    );

    let weather = result.unwrap();
    assert!(!weather.time.is_empty(), "Time should not be empty");
    assert!(
        weather.temperature_2m >= -30.0 && weather.temperature_2m <= 55.0,
        "Temperature {} out of plausible Canberra range",
        weather.temperature_2m
    );
    assert!(
        weather.wind_speed_10m >= 0.0,
        "Wind speed {} cannot be negative",
        weather.wind_speed_10m
    );
}

/// Live integration test for 7-day forecast against Open-Meteo API.
/// Marked #[ignore] so default tests run reliably offline.
/// Run explicitly with: cargo test -- --ignored
#[test]
#[ignore]
fn test_live_7day_weather() {
    let result = get_7day_weather();
    assert!(
        result.is_ok(),
        "Failed to fetch live 7-day weather: {:?}",
        result.err()
    );

    let daily = result.unwrap();
    assert_eq!(
        daily.time.len(),
        7,
        "Expected exactly 7 days of forecast dates"
    );
    assert_eq!(daily.temperature_2m_max.len(), 7, "Expected 7 max temps");
    assert_eq!(daily.temperature_2m_min.len(), 7, "Expected 7 min temps");
    assert_eq!(
        daily.precipitation_probability_max.len(),
        7,
        "Expected 7 rain probabilities"
    );

    for i in 0..7 {
        let date = &daily.time[i];
        let day_name = format_day_of_week(date);
        assert_ne!(
            day_name, "?",
            "Date {} failed to parse as valid weekday",
            date
        );

        assert!(
            daily.temperature_2m_min[i] <= daily.temperature_2m_max[i],
            "Min temp {} exceeds max temp {} on {}",
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            date
        );
        assert!(
            daily.precipitation_probability_max[i] <= 100,
            "Precipitation probability {} exceeds 100%",
            daily.precipitation_probability_max[i]
        );
    }
}
