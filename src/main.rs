use dotenv::dotenv;
use std::env;
use tonic::transport::ClientTlsConfig;
use tonic::{transport::Channel, Request};

pub mod kaiko {
    tonic::include_proto!("kaiko.benchmark_reference_rates");
}

use kaiko::benchmark_reference_rates_client::BenchmarkReferenceRatesClient;
use kaiko::SubscribeRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("KAIKO_API_KEY").expect("KAIKO_API_KEY not set");

    let tls_config = ClientTlsConfig::new().with_native_roots();
    let channel = Channel::from_static("https://gateway-v0-grpc.kaiko.ovh")
        .tls_config(tls_config)?
        .connect()
        .await?;

    println!("Connected to kaiko gRPC stream");

    let mut client =
        BenchmarkReferenceRatesClient::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut()
                .insert("x-api-key", api_key.parse().unwrap());
            Ok(req)
        });

    // https://docs.kaiko.com/kaiko-stream/rates-and-indices/digital-asset-rates/benchmark-reference-rates
    let request = tonic::Request::new(SubscribeRequest {
        index_codes: vec!["KK_BRR_BTCUSD".into()],
    });

    let mut stream = client.subscribe(request).await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Received data: {:?}", response);
    }

    Ok(())
}
