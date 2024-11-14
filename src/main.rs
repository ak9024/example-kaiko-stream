use dotenv::dotenv;
use std::env;
use tonic::transport::ClientTlsConfig;
use tonic::{transport::Channel, Request};

pub mod kaiko {
    tonic::include_proto!("kaiko.streaming");
}

use kaiko::market_data_client::MarketDataClient;
use kaiko::{Commodity, InstrumentCriteria, SubscribeRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("KAIKO_API_KEY").expect("KAIKO_API_KEY not set");

    let tls_config = ClientTlsConfig::new();
    let channel = Channel::from_static("https://gateway-v0-grpc.kaiko.ovh")
        .tls_config(tls_config)?
        .connect()
        .await?;

    println!("Connected to kaiko gRPC stream");

    let mut client = MarketDataClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("x-api-key", api_key.parse().unwrap());
        Ok(req)
    });

    // https://docs.kaiko.com/kaiko-stream/market-data/trades/tick-level-trades
    let request = tonic::Request::new(SubscribeRequest {
        instrument_criteria: Some(InstrumentCriteria {
            exchange: "cbse".into(),
            instrument_class: "spot".into(),
            code: "btc-usd".into(),
        }),
        commodities: vec![Commodity::SmucTrade.into()],
    });

    let mut stream = client.subscribe(request).await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Received data: {:?}", response);
    }

    Ok(())
}
