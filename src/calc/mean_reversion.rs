use rand_distr::{Distribution, Normal};

pub fn ou_process(
    initial_price: f64,
    theta: f64,
    mu: f64,
    sigma: f64,
    dt: f64,
    steps: usize,
) -> Vec<f64> {
    // Create a vector to store the prices
    let mut prices = vec![initial_price];

    // Initialize a random number generator
    let mut rng = rand::thread_rng();

    // Create the normal distribution (we assume this doesn't fail)
    let normal = Normal::new(0.0, 1.0).unwrap();

    // Simulate the OU process over the given number of steps
    for _ in 0..steps {
        // Get the latest price
        let last_price = *prices.last().unwrap();

        // Generate a random sample from the normal distribution
        let noise: f64 = normal.sample(&mut rng);

        // Calculate the change in price according to the OU formula
        let d_price = theta * (mu - last_price) * dt + sigma * noise * dt.sqrt();

        // Calculate the new price
        let new_price = last_price + d_price;

        // Append the new price to the vector
        prices.push(new_price);
    }

    prices
}
