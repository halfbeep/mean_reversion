![image](https://github.com/user-attachments/assets/84dc714e-a687-43c8-9198-9b7645c7c134)

This tool helps estimate how the price of Gold is likely to revert to its long-term average. It calculates the average price over a configurable number of periods (e.g., the last 36 hours), along with the realized volatility and the most recent price, using data fetched from Kraken for the PAX Gold price (PAXG), an on-chain proxy. The historical data for PAXG serves as a reliable and free 24/7 source, offering continuous returns for modeling

The mean reversion prediction is based on the Ornstein-Uhlenbeck (OU) process, a widely used model for mean-reverting behavior in financial markets. The speed of reversion, denoted by theta (θ), is adjusted based on empirical observations of how quickly prices have reverted in the past, ensuring the model is well-fitted to the data

<h2>$d X_t​=θ(μ−X_t​)dt+σd W_t​$</h2>

where:

- $dX_t$ is the change in the time series at time ttt
- $\theta$ is the speed of mean reversion. A higher θ\thetaθ means the series reverts to the mean faster
- $\mu$ is the long-term mean to which the series reverts
- $X_t$​ is the current value of the time series
- $dt$ is an infinitesimal increment of time
- $\sigma$ is the volatility of the process
- $dW_t$ is a Wiener process or Brownian motion term

