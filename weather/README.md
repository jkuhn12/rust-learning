# rust-weather

> A fast Rust CLI to get the current weather by US zip code. Single binary, no runtime needed.

A lightweight CLI powered by the [National Weather Service API](https://www.weather.gov/documentation/services-web-api).

## Features

- **Single binary** — no runtime, no dependencies to install
- **Accurate US forecasts** via the official National Weather Service API
- **Geocoding** via [Zippopotam.us](https://zippopotam.us) (zip → lat/lon)
- **Colorized, readable output** out of the box
- **Async/concurrent-friendly** architecture (Tokio + reqwest)
- **Type-safe JSON handling** with compile-time API contracts via Serde

## Installation

### From source

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone https://github.com/jeremykuhn/rust-weather.git
cd rust-weather
cargo build --release

# The binary is now available at:
./target/release/rust-weather
```

### Using cargo

```bash
cargo install --path .
```

## Usage

```bash
rust-weather <ZIP_CODE>
```

### Example

```bash
$ rust-weather 90210

→ Looking up weather for zip code 90210...
✓ Found location: Beverly Hills, CA (34.0901, -118.4065)
✓ NWS forecast office covers: Beverly Hills, CA

Current Weather
  Period: This Afternoon
  Temperature: 72°F
  Conditions: Mostly Sunny
  Wind: 5 mph SW

Sunny, with a high near 72. Southwest wind around 5 mph.
```

## How it works

1. **Geocode** the zip code via Zippopotam.us to get latitude/longitude
2. Query the NWS `/points` endpoint to discover the local forecast grid
3. Fetch the detailed forecast and display the current period

## Tech Stack

| Crate | Purpose |
|-------|---------|
| [tokio](https://tokio.rs) | Async runtime |
| [reqwest](https://github.com/seanmonstar/reqwest) | HTTP client |
| [serde](https://serde.rs) | JSON deserialization |
| [clap](https://github.com/clap-rs/clap) | CLI argument parsing |
| [anyhow](https://github.com/dtolnay/anyhow) | Ergonomic error handling |
| [colored](https://github.com/mackwic/colored) | Terminal colors |

## Why Rust?

See [`RUST_VS_OTHERS.md`](RUST_VS_OTHERS.md) for a detailed comparison of how this project would differ if written in Python or Go, covering memory management, error handling, async performance, and type safety.

## License

MIT
