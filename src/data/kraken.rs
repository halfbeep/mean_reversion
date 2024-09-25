use anyhow::Result;
use chrono::{NaiveDateTime, TimeZone, Utc};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

#[path = "../util/rounding.rs"]
mod rounding;
use rounding::round_to_period;

// Define a struct to hold the response from the Kraken API
#[derive(Deserialize, Debug)]
struct KrakenApiResponse {
    result: KrakenResult,
}

#[derive(Deserialize, Debug)]
struct KrakenResult {
    #[serde(rename = "last")]
    _last: Value, // We don't need this field, so use `Value` to skip it
    #[serde(flatten)]
    ohlc: HashMap<String, Vec<Vec<Value>>>,
}

pub async fn fetch_data_and_update_map(
    results_map: &Arc<RwLock<HashMap<NaiveDateTime, (Option<f64>, Option<f64>)>>>, // Borrow the Arc
    time_period: &str, // Pass as a reference
) {
    match get_kraken_data(time_period).await {
        Ok(new_data) => {
            debug!("Fetched Kraken data: {:?}", new_data);

            {
                // Hold the lock only when updating the map, to avoid deadlocks
                let mut map = results_map.write().unwrap();
                debug!("Acquired write lock for updating map with new data");

                for (timestamp, price) in new_data {
                    let rounded_timestamp = round_to_period(timestamp, time_period);
                    map.entry(rounded_timestamp)
                        .and_modify(|e| e.0 = Some(price))
                        .or_insert((Some(price), None));
                }
                debug!("Updated results map with new data: {:?}", map);

                // Truncate results_map to the most recent NO_OF_PERIODS length
                let no_of_periods: usize = env::var("NO_OF_PERIODS")
                    .unwrap_or("100".to_string())
                    .parse()
                    .expect("NO_OF_PERIODS must be a valid integer");

                let mut timestamps: Vec<_> = map.keys().cloned().collect();
                timestamps.sort(); // Sorting in ascending order, oldest to newest

                if timestamps.len() > no_of_periods {
                    let excess_count = timestamps.len() - no_of_periods;
                    for timestamp in &timestamps[..excess_count] {
                        map.remove(timestamp);
                    }
                }

                debug!("Map size after trimming: {}", map.len());
            } // Write lock released here automatically when map goes out of scope
        }
        Err(e) => {
            println!("Failed to fetch Kraken data: {}", e);
        }
    }
}

// Function to fetch Kraken OHLC data
async fn get_kraken_data(time_period: &str) -> Result<Vec<(NaiveDateTime, f64)>, anyhow::Error> {
    let asset_id = "PAXGUSD";

    // Convert time_period to the correct interval in minutes for Kraken API
    let interval_minutes = match time_period {
        "minute" => 1, // 1 minute
        "hour" => 60,  // 60 minutes
        "day" => 1440, // 1440 minutes (24 hours)
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported time period provided for Kraken data"
            ))
        } // Return an error for unsupported time periods (like seconds)
    };

    // Construct the actual URL
    let url = format!(
        "https://api.kraken.com/0/public/OHLC?pair={}&interval={}",
        asset_id, interval_minutes
    );

    let client = Client::new();

    // Make the request to API
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<KrakenApiResponse>()
        .await?;

    // Extract OHLC data
    let ohlc_data = response
        .result
        .ohlc
        .get(asset_id)
        .ok_or_else(|| anyhow::anyhow!("No OHLC data found for the specified pair"))?;

    // Parse the average of OHLC into vec(NaiveDateTime, f64)
    let parsed_ohlc: Vec<(NaiveDateTime, f64)> = ohlc_data
        .iter()
        .filter_map(|ohlc| {
            if ohlc.len() < 5 {
                return None; // Ensure OHLC has enough fields (timestamp, o, h, l, c)
            }

            debug!(
                "Timestamp & OHLC: {} {} {} {} {}",
                &ohlc[0], &ohlc[1], &ohlc[2], &ohlc[3], &ohlc[4]
            );

            // ohlc[0] contains the timestamp, which may be an integer or string
            let timestamp = match &ohlc[0] {
                Value::Number(n) => n.as_i64().unwrap_or(0),
                Value::String(s) => s.parse::<i64>().unwrap_or(0),
                _ => return None,
            };

            // Convert timestamp to NaiveDateTime using Utc and then naive_utc
            let datetime = Utc.timestamp_opt(timestamp, 0).single()?.naive_utc();

            // ohlc[1] to ohlc[4] contain the open, high, low, and close prices
            let open_price = match &ohlc[1] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => return None,
            };

            let high_price = match &ohlc[2] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => return None,
            };

            /*
            let low_price = match &ohlc[3] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => return None,
            };
            */

            let close_price = match &ohlc[4] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => return None,
            };

            // Calculate the close weighted average price (ohcc/4)
            let average_price = (open_price + high_price + close_price + close_price) / 4.0;

            Some((datetime, average_price))
        })
        .collect();

    Ok(parsed_ohlc)
}
