#[cfg(test)]
mod tests {
    use crate::mean_reversion::ou_process;

    #[test]
    fn test_ou_process_output_length() {
        // Define test parameters
        let initial_price = 100.0;
        let theta = 0.15;
        let mu = 100.0;
        let sigma = 0.2;
        let dt = 1.0 / 252.0;
        let steps = 60;

        // Run the OU process
        let prices = ou_process(initial_price, theta, mu, sigma, dt, steps);

        // Test: The output vector length should be steps + 1 (initial price + 60 steps)
        assert_eq!(prices.len(), steps + 1, "Output length should be steps + 1");
    }

    #[test]
    fn test_ou_process_initial_price() {
        // Define test parameters
        let initial_price = 50.0;
        let theta = 0.1;
        let mu = 50.0;
        let sigma = 0.2;
        let dt = 1.0 / 252.0;
        let steps = 100;

        // Run the OU process
        let prices = ou_process(initial_price, theta, mu, sigma, dt, steps);

        // Test: The first price in the output should match the initial price
        assert_eq!(
            prices[0], initial_price,
            "The first price should match the initial price"
        );
    }

    #[test]
    fn test_ou_process_mean_reversion_trend() {
        // Define test parameters
        let initial_price = 80.0; // Price lower than the mean
        let theta = 0.2;
        let mu = 100.0; // Mean reversion to 100
        let sigma = 0.1;
        let dt = 1.0 / 252.0;
        let steps = 100;

        // Run the OU process
        let prices = ou_process(initial_price, theta, mu, sigma, dt, steps);

        // Test: The final price should be closer to the mean (mu) than the initial price
        let final_price = *prices.last().unwrap();
        assert!(
            (final_price - mu).abs() < (initial_price - mu).abs(),
            "The final price should be closer to the mean"
        );
    }
}
