use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn get_latest_vol_price(
    results_map: &Arc<RwLock<HashMap<NaiveDateTime, (Option<f64>, Option<f64>)>>>,
) -> Option<f64> {
    // Acquire a read lock on the results_map
    let map = results_map.read().unwrap();

    // Get the latest timestamp by sorting the keys and return only the vol price (2nd element)
    map.keys().max().and_then(|latest_timestamp| {
        map.get(latest_timestamp)
            .and_then(|(_, latest_vol_price)| *latest_vol_price) // Get only the 2nd element (vol price)
    })
}

pub fn get_mean_vol_price(
    results_map: &Arc<RwLock<HashMap<NaiveDateTime, (Option<f64>, Option<f64>)>>>,
) -> Option<f64> {
    // Acquire a read lock on the results_map
    let map = results_map.read().unwrap();

    // Collect all the valid vol prices (i.e., not None)
    let vol_prices: Vec<f64> = map
        .values()
        .filter_map(|(_, vol_price)| *vol_price) // Get only the 2nd element (vol price) and filter out None
        .collect();

    // If there are no valid vol prices, return None
    if vol_prices.is_empty() {
        return None;
    }

    // Calculate the mean of the vol prices
    let sum: f64 = vol_prices.iter().sum();
    let mean = sum / vol_prices.len() as f64;

    Some(mean)
}
