I use this tool to gauge how Gold will revert to its long term average. Using the average of the last NO_OF_PERIODS (say 36 hrs), realised volatility and most recent price are fetched for the PAX Gold proxy from Kraken. PAXG historical data is an easy (and free) 24/7 data source offering continuous returns. The model follows the Ornstein-Uhlenbeck (OU) process for mean reversion. Theta is the speeed of reversion which I adjust from emperical, realised reversions 

$d X_t​=θ(μ−X_t​)dt+σd W_t​$

where:

- $dX_t$ is the change in the time series at time ttt
- $\theta$ is the speed of mean reversion. A higher θ\thetaθ means the series reverts to the mean faster
- $\mu$ is the long-term mean to which the series reverts
- $X_t$​ is the current value of the time series
- $dt$ is an infinitesimal increment of time
- $\sigma$ is the volatility of the process
- $dW_t$ is a Wiener process or Brownian motion term
