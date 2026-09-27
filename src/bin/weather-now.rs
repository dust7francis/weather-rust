use weather_rust::{current_local_time, get_current_weather};

fn main() {
    match get_current_weather() {
        Ok(weather) => {
            println!("☀️ Canberra Current Weather ☀️");
            println!("Time (Canberra): {}", weather.time);
            println!("Temperature: {:.1}°C", weather.temperature_2m);
            println!("Wind speed: {:.1} km/h", weather.wind_speed_10m);
        }
        Err(error) => {
            eprintln!("Error fetching weather data: {error}");
            std::process::exit(1);
        }
    }

    // This is the computer's local time and is useful for troubleshooting.
    // Uncomment if desired:
    // println!("Local computer time: {}", current_local_time());
    let _ = current_local_time;
}
