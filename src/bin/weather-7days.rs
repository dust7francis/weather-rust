use chrono::NaiveDate;
use weather_rust::get_7day_weather;

fn main() {
    match get_7day_weather() {
        Ok(weather) => {
            println!("☀️ Canberra 7-Day Weather Forecast ☀️");
            println!();
            println!(
                "{:<12} {:<12} {:>8} {:>8} {:>8}",
                "Date", "Day", "Min", "Max", "Rain"
            );

            for i in 0..weather.time.len() {
                let date = &weather.time[i];
                let day = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map(|d| d.format("%A").to_string())
                    .unwrap_or_else(|_| "?".to_string());

                println!(
                    "{:<12} {:<12} {:>6.1}°C {:>6.1}°C {:>7}%",
                    date,
                    day,
                    weather.temperature_2m_min[i],
                    weather.temperature_2m_max[i],
                    weather.precipitation_probability_max[i]
                );
            }
        }
        Err(error) => {
            eprintln!("Error fetching weather data: {error}");
            std::process::exit(1);
        }
    }
}
