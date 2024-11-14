use dotenv::dotenv;
use std::env;
use tonic::metadata::MetadataValue;
use tonic::transport::ClientTlsConfig;
use tonic::{transport::Channel, Request};

use kaikosdk::stream_index_service_v1_client::StreamIndexServiceV1Client;
use kaikosdk::StreamIndexServiceRequestV1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("KAIKO_API_KEY").expect("KAIKO_API_KEY not set");
    let token: MetadataValue<_> = format!("Bearer {}", api_key).parse()?;

    let tls_config = ClientTlsConfig::new().with_native_roots();
    let channel = Channel::from_static("https://gateway-v0-grpc.kaiko.ovh")
        .tls_config(tls_config)?
        .connect()
        .await?;

    println!("Connected to kaiko gRPC stream");

    let mut client =
        StreamIndexServiceV1Client::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        });

    // https://docs.kaiko.com/kaiko-stream/rates-and-indices/digital-asset-rates/benchmark-reference-rates
    let request = tonic::Request::new(StreamIndexServiceRequestV1 {
        index_code: "KK_BRR_BTCUSD".into(),
        ..Default::default()
    });

    let mut stream = client.subscribe(request).await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Received data: {:?}", response);
    }

    Ok(())
}
