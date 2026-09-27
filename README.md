# Canberra Weather - Rust

Two small native Rust command-line programs using the Open-Meteo API:

- `weather-now` — current Canberra temperature and wind speed
- `weather-7days` — seven-day Canberra forecast

## Requirements

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Restart your terminal after installation, then check:

```bash
rustc --version
cargo --version
```

## Build

From this directory:

```bash
cargo build --release
```

The native executables will be:

```text
target/release/weather-now
target/release/weather-7days
```

Run them:

```bash
./target/release/weather-now
./target/release/weather-7days
```

No Python, `uv`, virtual environment, or Rust runtime is required on the target machine after compilation.

## Optional: copy executables to a local bin directory

For example:

```bash
mkdir -p ~/bin
cp target/release/weather-now ~/bin/
cp target/release/weather-7days ~/bin/
```

Then, if `~/bin` is in your PATH:

```bash
weather-now
weather-7days
```

## Sample current-weather output

```text
☀️ Canberra Current Weather ☀️
Time (Canberra): 2026-09-27T...
Temperature: 22.4°C
Wind speed: 11.5 km/h
```

## Sample 7-day output

```text
☀️ Canberra 7-Day Weather Forecast ☀️

Date         Day              Min      Max     Rain
2026-09-27   Sunday          8.2°C   22.4°C      10%
2026-09-28   Monday          9.1°C   24.1°C      20%
...
```

## Cross-compilation

For a macOS Apple Silicon machine, the normal release build is:

```bash
cargo build --release
```

which creates an executable for the current machine.

For an Intel Mac, Linux machine, or other target, install the appropriate Rust target and cross-compilation toolchain. For example, an Apple Silicon Mac can build an Intel macOS executable with:

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

The resulting executable is:

```text
target/x86_64-apple-darwin/release/weather-now
target/x86_64-apple-darwin/release/weather-7days
```

## API

Weather data is provided by Open-Meteo:

https://open-meteo.com/

The programs use Canberra coordinates:

- Latitude: `-35.2835`
- Longitude: `149.1281`
- Time zone: `Australia/Sydney`
