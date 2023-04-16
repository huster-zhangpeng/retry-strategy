# retry-strategy

A better utilities for asynchronous Future. The raw tokio::timeout can only complete the timeout task setting once, but more often, such as a tcp connection or a http request, multiple retries are required, and the timeout waiting time for each retry is different. Strategy to set the duration of each timeout, until all timeout opportunities are exhausted, if there is still no result, it is the final timeout.

# Installation

Add dependencies to your `Cargo.toml`:

```
[dependencies]
retry-strategy = "0.1"
```

# Examples 

```
use retry_strategy::retry;

let tcp_connection = retry(
    Opportunity(vec![100, 200, 300]), 
    |_n| TcpStream::connect("127.0.0.1:8080")
).await?;
```


