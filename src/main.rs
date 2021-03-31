use async_std::prelude::*;
use async_std::stream;
use chrono::prelude::*;
use clap::Clap;
use std::time::Duration;
use xactor::*;
use yahoo_finance_api as yahoo;

mod utils;

use utils::{fetch_ticker_data, max, min, n_window_sma, price_diff};

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Claus Matzinger",
    about = "Milestone 2: working with actors"
)]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

#[message]
#[derive(Debug, Default, Clone)]
struct Quotes {
    pub symbol: String,
    pub quotes: Vec<yahoo::Quote>,
}

#[message]
#[derive(Debug, Clone)]
struct QuoteRequest {
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

pub struct StockDataDownloader;

#[async_trait::async_trait]
impl Handler<QuoteRequest> for StockDataDownloader {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: QuoteRequest) {
        println!("StockDataDownloader Handler");
        let symbol = msg.symbol.clone();
        // 1h interval works for larger time periods as well (months/years)
        dbg!(&symbol);
        let data = match fetch_ticker_data(msg.symbol, msg.from, msg.to, String::from("1h")).await {
            Ok(quotes) => Quotes {
                symbol: symbol.clone(),
                quotes,
            },
            Err(e) => {
                eprintln!("Ignoring API error for symbol '{}': {}", symbol, e);
                Quotes {
                    symbol: symbol.clone(),
                    quotes: vec![],
                }
            }
        };
        if let Err(e) = Broker::from_registry().await.unwrap().publish(data) {
            eprint!("{}", e);
        }
    }
}

#[async_trait::async_trait]
impl Actor for StockDataDownloader {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("Stock Data Downloader Actor");
        ctx.subscribe::<QuoteRequest>().await
    }
}

struct StockDataProcessor;

#[async_trait::async_trait]
impl Handler<Quotes> for StockDataProcessor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, mut msg: Quotes) {
        println!("StockDataProcessor Handler");
        let data = msg.quotes.as_mut_slice();
        if !data.is_empty() {
            // ensure that the data is sorted by time (asc)
            data.sort_by_cached_key(|k| k.timestamp);

            let last_date = Utc.timestamp(data.last().unwrap().timestamp as i64, 0);

            let close_prices: Vec<f64> = data.iter().map(|q| q.close).collect();
            let last_price: f64 = *close_prices.last().unwrap();
            let period_min = min(&close_prices).await.unwrap_or(0.0);
            let period_max = max(&close_prices).await.unwrap_or(0.0);

            let (_, pct_change) = price_diff(&close_prices).await.unwrap_or((0.0, 0.0));
            let sma = n_window_sma(30, &close_prices).await.unwrap_or_default();

            println!(
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                last_date.to_rfc3339(),
                msg.symbol,
                last_price,
                pct_change * 100.0,
                period_min,
                period_max,
                sma.last().unwrap_or(&0.0)
            );
        }
    }
}

#[async_trait::async_trait]
impl Actor for StockDataProcessor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("Stock Data Processor Actor");
        ctx.subscribe::<Quotes>().await
    }
}

#[xactor::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let symbols: Vec<String> = opts.symbols.split(',').map(|s| s.to_owned()).collect();

    // Start actors
    let downloader = Supervisor::start(|| StockDataDownloader);
    let processor = Supervisor::start(|| StockDataProcessor);
    let _ = downloader.join(processor).await;

    // CSV header
    println!("period start,symbol,price,change %,min,max,30d avg");
    let mut interval = stream::interval(Duration::from_secs(10));
    'outer: while interval.next().await.is_some() {
        let now = Utc::now(); // Period end for this fetch
        for symbol in &symbols {
            println!("Processing symbol {}", symbol);
            if let Err(e) = Broker::from_registry().await?.publish(QuoteRequest {
                symbol: symbol.clone(),
                from,
                to: now,
            }) {
                eprint!("{}", e);
                break 'outer;
            }
        }
    }
    Ok(())
}
