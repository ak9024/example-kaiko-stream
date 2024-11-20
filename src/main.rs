use chrono::DateTime;
use dotenv::dotenv;
use kaikosdk::{
    stream_index_service_v1_client::StreamIndexServiceV1Client, StreamIndexServiceRequestV1,
    StreamIndexServiceResponseV1,
};
use std::collections::VecDeque;
use std::env;
use tonic::metadata::MetadataValue;
use tonic::transport::ClientTlsConfig;
use tonic::{transport::Channel, Request};

#[derive(Debug, Clone, Copy)]
enum TimeFilter {
    Hourly,
    Daily,
}

#[derive(Debug, Clone)]
struct IndexData {
    index_price: f64,
    timestamp: u64,
}

impl IndexData {
    fn from_kaiko_response(response: &StreamIndexServiceResponseV1) -> Option<Self> {
        let first_percentage = response.percentages.first()?;
        let ts_event = response.ts_event.as_ref()?;

        Some(IndexData {
            index_price: first_percentage.price,
            timestamp: ts_event.seconds as u64,
        })
    }
}

#[derive(Debug, Clone)]
struct PriceData {
    ask_price: f64,
    last_traded_price: f64,
    index_data: IndexData,
}

impl PriceData {
    fn from_kaiko_response(response: &StreamIndexServiceResponseV1) -> Option<Self> {
        let index_data = IndexData::from_kaiko_response(response)?;
        let spread = 0.001;
        let mid_price = index_data.index_price;

        Some(PriceData {
            ask_price: mid_price * (1.0 + spread),
            last_traded_price: mid_price,
            index_data,
        })
    }
}

#[derive(Debug)]
struct PriceStream {
    prices: VecDeque<PriceData>,
    max_window_size: usize,
    time_filter: TimeFilter,
}

impl PriceStream {
    fn new(max_window_size: usize, time_filter: TimeFilter) -> Self {
        PriceStream {
            prices: VecDeque::with_capacity(max_window_size),
            max_window_size,
            time_filter,
        }
    }

    fn add_price(&mut self, price_data: PriceData) {
        if self.prices.len() >= self.max_window_size {
            self.prices.pop_front();
        }
        self.prices.push_back(price_data);
    }

    fn calculate_mark_price(&self) -> Option<f64> {
        let latest = self.prices.back()?;
        let index_price = latest.index_data.index_price;

        let upper_bound = latest.last_traded_price * 1.0025;
        let lower_bound = latest.last_traded_price * 0.9975;

        let fair_price = latest.ask_price.min(upper_bound).max(lower_bound);

        Some((fair_price + index_price) / 2.0)
    }

    fn calculate_funding_rate(&self) -> Option<f64> {
        let latest = self.prices.back()?;
        let mark_price = self.calculate_mark_price()?;
        let index_price = latest.index_data.index_price;

        let multiplier = match self.time_filter {
            TimeFilter::Hourly => 1.0,
            TimeFilter::Daily => 24.0,
        };

        Some((mark_price / index_price - 1.0) * multiplier)
    }
}

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

    println!("Connected to Kaiko gRPC stream");

    let mut client =
        StreamIndexServiceV1Client::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        });

    let request = tonic::Request::new(StreamIndexServiceRequestV1 {
        index_code: "KK_BRR_BTCUSD".into(),
        ..Default::default()
    });

    let mut stream = client.subscribe(request).await?.into_inner();
    let mut hourly_stream = PriceStream::new(10, TimeFilter::Hourly);
    let mut daily_stream = PriceStream::new(24, TimeFilter::Daily);

    while let Some(response) = stream.message().await? {
        if let Some(price_data) = PriceData::from_kaiko_response(&response) {
            let dt = DateTime::from_timestamp(price_data.index_data.timestamp as i64, 0).unwrap();

            hourly_stream.add_price(price_data.clone());
            daily_stream.add_price(price_data.clone());

            println!("Timestamp: {}", dt.format("%Y-%m-%d %H:%M:%S"));
            println!("Index Price: {}", price_data.index_data.index_price);
            if let Some(mark_price) = hourly_stream.calculate_mark_price() {
                println!("Mark Price: {}", mark_price);

                if let Some(hourly_rate) = hourly_stream.calculate_funding_rate() {
                    println!("Hourly Funding Rate: {:.6}%", hourly_rate * 100.0);
                }
                if let Some(daily_rate) = daily_stream.calculate_funding_rate() {
                    println!("Daily Funding Rate: {:.6}%", daily_rate * 100.0);
                }
            }
            println!("---");
        }
    }

    Ok(())
}
