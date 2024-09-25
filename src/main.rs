use anyhow::Result;
use chrono::{Duration, NaiveDateTime, Utc};
use dotenv::dotenv;
use log::debug;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

#[path = "./data/kraken.rs"]
mod kraken;
use kraken::fetch_data_and_update_map;

#[path = "./util/rounding.rs"]
mod rounding;
use rounding::round_to_period;

#[path = "./util/price.rs"]
mod price;
use price::get_latest_vol_price;
use price::get_mean_vol_price;

#[path = "./calc/mean_reversion.rs"]
mod mean_reversion;
use mean_reversion::ou_process;

#[cfg(test)]
#[path = "./calc/mean_reversion_test.rs"]
mod mean_reversion_test;

#[path = "./calc/calculate_volatility.rs"]
mod calculate_volatility;
use calculate_volatility::calculate_volatility;

#[cfg(test)]
#[path = "./calc/calculate_volatility_test.rs"]
mod calculate_volatility_test;

type ResultsMap = Arc<RwLock<HashMap<NaiveDateTime, (Option<f64>, Option<f64>)>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger once at the start of the program
    if env_logger::try_init().is_err() {
        eprintln!("Logger was already initialized");
    }
    dotenv().ok();

    // Load the number of periods from the .env file
    let no_of_periods: usize = env::var("NO_OF_PERIODS")
        .unwrap_or("100".to_string()) // Default to 100 periods if not set
        .parse()
        .expect("NO_OF_PERIODS must be a valid integer");

    // Check that NO_OF_PERIODS is in a reasonable range
    if no_of_periods == 0 || no_of_periods >= 741 {
        return Err(anyhow::anyhow!(
            "NO_OF_PERIODS must be greater than 0 and less than 741"
        ));
    }

    // Load the number theta from the .env file
    let speed_theta: f64 = env::var("SPEED_THETA")
        .unwrap_or_else(|_| "0.5".to_string()) // Default to 0.5 if not set or invalid
        .parse()
        .expect("SPEED_THETA must be a valid float");

    // Check that speed_theta is in a reasonable range
    if speed_theta <= 0.0 || speed_theta > 5.0 {
        return Err(anyhow::anyhow!(
            "SPEED_THETA must be greater than 0 and less than or equal to 5"
        ));
    }

    // Default to hour if period is absent
    let time_period = env::var("TIME_PERIOD").unwrap_or("hour".to_string());
    if !["second", "minute", "hour", "day"].contains(&time_period.as_str()) {
        return Err(anyhow::anyhow!(
            "TIME_PERIOD must be one of: 'second', 'minute', 'hour', or 'day'."
        ));
    }

    debug!(
        "No of periods {}, Time period {}",
        no_of_periods, time_period
    );

    // Determine the duration based on TIME_PERIOD
    let time_duration = match time_period.as_str() {
        "second" => Duration::seconds(1),
        "minute" => Duration::minutes(1),
        "hour" => Duration::hours(1),
        "day" => Duration::days(1),
        _ => Duration::hours(1), // Default to 'hour' if the provided value is invalid
    };

    // Initialize the starting timestamp (now - time_period)
    let mut current_timestamp = Utc::now().naive_utc();

    // Initialize a results_map with an Option<f64> placeholder for prices
    let results_map: ResultsMap = Arc::new(RwLock::new(HashMap::new()));
    // Fill in initial timestamps with None values
    {
        let mut map = results_map.write().map_err(|e| {
            eprintln!("Error acquiring write lock on results_map: {:?}", e);
            anyhow::anyhow!("Lock poisoning detected.")
        })?;

        for _ in 0..no_of_periods {
            let rounded_timestamp = round_to_period(current_timestamp, &time_period);
            map.insert(rounded_timestamp, (None, None));
            current_timestamp = current_timestamp - time_duration;
            debug!("{}", rounded_timestamp);
        }
    }

    fetch_data_and_update_map(&results_map, &time_period).await;

    let sd = calculate_volatility(&results_map, no_of_periods);

    let avg_price = get_mean_vol_price(&results_map);

    let latest = get_latest_vol_price(&results_map);

    debug!(
        "SD: {}, latest {}, avg {},  theta {}",
        sd.unwrap(),
        latest.unwrap(),
        avg_price.unwrap(),
        speed_theta
    );

    // Define the parameters for the OU process
    let initial_price = latest.unwrap(); // Starting price
    let mu = avg_price.unwrap(); // Long-term mean
    let sigma = sd.unwrap(); // Volatility
    let dt = 1.0 / 60.0; // Time step (daily if 252 trading days in a year)
    let steps = 60; // Number of time steps to simulate

    let theta = speed_theta; // Mean reversion speed

    // Run the OU process simulation
    let prices: Vec<f64> = ou_process(initial_price, theta, mu, sigma, dt, steps);

    // Print the generated prices
    for (i, price) in prices.iter().enumerate() {
        println!("Period: {}: {:.2}", i, price);
    }

    // Return Ok(()) to indicate successful execution
    Ok(())
}
