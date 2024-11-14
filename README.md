# kaiko_stream

### Sequence Diagram 

```
title Kaiko Data Stream

participant Rust Client
participant gRPC Channel
participant Kaiko API

note over Rust Client: Application startup
Rust Client->gRPC Channel: Establish connection
gRPC Channel->Kaiko API: Authenticate with API key
Kaiko API-->gRPC Channel: Authentication successful

note over Rust Client: Send subscription request
Rust Client->gRPC Channel: SubscribeRequest(market: "btc-usdt", data_type: "trades")
gRPC Channel->Kaiko API: Forward subscription request
Kaiko API-->gRPC Channel: Stream data

loop Real-time data stream
    Kaiko API-->gRPC Channel: MarketDataResponse(price, volume, timestamp)
    gRPC Channel-->Rust Client: Stream data
    Rust Client->Rust Client: Process and print data
end

note over Rust Client: Connection ends on error or cancellation
Rust Client->gRPC Channel: Close connection
gRPC Channel->Kaiko API: Terminate subscription
```
