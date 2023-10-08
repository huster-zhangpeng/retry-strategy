# retry-strategy

A better asynchronous retry tool based on Tokio. The original `Tokio::timeout` function ends after a single timeout, but a single timeout may be affected by random factors. For example, establishing a connection or making an HTTP request often requires three timeout retries. If all retries fail, it is considered a final timeout. 

`Retry-strategy` provides rich and simple timeout settings, allowing you to easily and flexibly set the number of retries, the duration of each timeout, and even more complex strategies. We hope it brings convenience to you.

# Installation

Add dependencies to your `Cargo.toml`:

```
[dependencies]
retry-strategy = "0.2"
```

# Examples 

```
use retry_strategy::prelude::*;

let tcp_connection = retry(
    vec![100.ms(), 200.ms(), 300.ms()], 
    |_n| TcpStream::connect("127.0.0.1:8080")
).await?;
```


