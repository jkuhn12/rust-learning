use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use serde::Deserialize;

/// A simple CLI to get the current weather by US zip code.
/// It uses zippopotam.us for geocoding and the National Weather Service (NWS) API for forecasts.
#[derive(Parser)]
#[command(name = "rust-weather")]
#[command(about = "Get the current weather for a US zip code")]
struct Cli {
    /// The US zip code to look up
    zip: String,
}

// ─── Geocoding (Zip → Lat/Lon) ───

#[derive(Deserialize, Debug)]
struct ZippoResponse {
    places: Vec<Place>,
}

#[derive(Deserialize, Debug)]
struct Place {
    latitude: String,
    longitude: String,
    #[serde(rename = "place name")]
    place_name: String,
    #[serde(rename = "state abbreviation")]
    state: String,
}

async fn geocode_zip(zip: &str) -> Result<(f64, f64, String, String)> {
    let url = format!("https://api.zippopotam.us/us/{}", zip);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to send geocoding request to {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("Zip code '{}' not found or geocoding service unavailable", zip);
    }

    let data: ZippoResponse = resp
        .json()
        .await
        .context("Failed to parse geocoding response")?;

    let place = data.places.into_iter().next().context("No location data found for zip code")?;

    let lat: f64 = place.latitude.parse().context("Invalid latitude format")?;
    let lon: f64 = place.longitude.parse().context("Invalid longitude format")?;

    Ok((lat, lon, place.place_name, place.state))
}

// ─── NWS API: Points ───

#[derive(Deserialize, Debug)]
struct PointsResponse {
    properties: PointsProperties,
}

#[derive(Deserialize, Debug)]
struct PointsProperties {
    forecast: String,
    #[serde(rename = "relativeLocation")]
    relative_location: RelativeLocation,
}

#[derive(Deserialize, Debug)]
struct RelativeLocation {
    properties: RelativeLocationProperties,
}

#[derive(Deserialize, Debug)]
struct RelativeLocationProperties {
    city: String,
    state: String,
}

async fn get_points(lat: f64, lon: f64) -> Result<PointsProperties> {
    let url = format!("https://api.weather.gov/points/{:.4},{:.4}", lat, lon);
    let client = reqwest::Client::builder()
        .user_agent("rust-weather-cli/0.1.0 (learning@rust.dev)")
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to send request to {}", url))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("NWS points API returned {}: {}", status, text);
    }

    let data: PointsResponse = resp
        .json()
        .await
        .context("Failed to parse NWS points response")?;

    Ok(data.properties)
}

// ─── NWS API: Forecast ───

#[derive(Deserialize, Debug)]
struct ForecastResponse {
    properties: ForecastProperties,
}

#[derive(Deserialize, Debug)]
struct ForecastProperties {
    periods: Vec<ForecastPeriod>,
}

#[derive(Deserialize, Debug)]
struct ForecastPeriod {
    name: String,
    temperature: i32,
    #[serde(rename = "temperatureUnit")]
    temperature_unit: String,
    #[serde(rename = "shortForecast")]
    short_forecast: String,
    #[serde(rename = "detailedForecast")]
    detailed_forecast: String,
    #[serde(rename = "windSpeed")]
    wind_speed: String,
    #[serde(rename = "windDirection")]
    wind_direction: String,
}

async fn get_forecast(forecast_url: &str) -> Result<ForecastPeriod> {
    let client = reqwest::Client::builder()
        .user_agent("rust-weather-cli/0.1.0 (learning@rust.dev)")
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(forecast_url)
        .send()
        .await
        .with_context(|| format!("Failed to send request to {}", forecast_url))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("NWS forecast API returned {}: {}", status, text);
    }

    let data: ForecastResponse = resp
        .json()
        .await
        .context("Failed to parse NWS forecast response")?;

    data.properties
        .periods
        .into_iter()
        .next()
        .context("No forecast periods returned")
}

// ─── Main ───

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    println!("{} Looking up weather for zip code {}...", "→".cyan(), args.zip.bold());

    // Step 1: Convert zip code to latitude/longitude
    let (lat, lon, city, state) = geocode_zip(&args.zip).await?;
    println!(
        "{} Found location: {}, {} ({:.4}, {:.4})",
        "✓".green(),
        city.bold(),
        state,
        lat,
        lon
    );

    // Step 2: Get NWS grid point metadata
    let points = get_points(lat, lon).await?;
    println!(
        "{} NWS forecast office covers: {}, {}",
        "✓".green(),
        points.relative_location.properties.city.bold(),
        points.relative_location.properties.state
    );

    // Step 3: Get the forecast
    let current = get_forecast(&points.forecast).await?;

    // Step 4: Print the weather
    println!();
    println!("{}", "Current Weather".bold().underline());
    println!("  {}: {}", "Period".bold(), current.name);
    println!(
        "  {}: {}°{}",
        "Temperature".bold(),
        current.temperature.to_string().yellow(),
        current.temperature_unit
    );
    println!("  {}: {}", "Conditions".bold(), current.short_forecast);
    println!("  {}: {} {}", "Wind".bold(), current.wind_speed, current.wind_direction);
    println!();
    println!("{}", current.detailed_forecast);

    Ok(())
}
