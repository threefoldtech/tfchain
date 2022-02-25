# Tft price

We use the TFT Price to calculate how much TFT the user is due for a specific workload over time. On chain, there is no easy way to ad-hoc request the TFT Price, this is why we are using the [substrate offchain worker](https://docs.substrate.io/v3/concepts/off-chain-features/#:~:text=Off%2DChain%20Worker%20(OCW),%2Dchain%20data%2C%20etc.) to periodically fetch the price and store it in the runtime storage so that this is accesible by the runtime.

The Off-Chain worker is executed every 10 blocks, meaning every 1 minute. The worker does a HTTP request to following endpoint 

- https://min-api.cryptocompare.com/data/price?fsym=3ft&tsyms=USD 

It has 2 main storage maps:

- Current price
- Average price

It stores the current price every 1 minute, the average price gets stored every 100 blocks, meaning 10 minutes. It calculates the average price based on a ringbuffer that has a size of max 1440 items. An item to this ringbuffer is appended every 1 minute (when the price is fetched). When the amount of items in the buffer exceed 1440, it takes the first (oldest) item out and places the new value and the end. 

To calculate the average, it sums up all the items in the ringbuffer and divides it by the amount of values in the buffer. This yields the average price. It does this operation every 1 minute and stores the result in the Average price storage map.