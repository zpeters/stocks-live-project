use yahoo_finance_api as yahoo;
use std::error::Error;
use clap::{Arg, App};

fn get_quote(ticker: &str, period: &str) -> Result<String, Box<dyn Error>> {
    let provider = yahoo::YahooConnector::new();

    match provider.get_quote_range(ticker, "1m", period) {
        Ok(response) => {
            match response.quotes() {
                Ok(quotes) => {
                    Ok(format!("{} quotes for {} are: {:?}", ticker, period, quotes))
                },
                Err(e) => {
                    eprintln!("Inner error");
                    return Err(Box::new(e))}
                    ,
            }
        },
        Err(e) => {
            eprintln!("Ticker '{}' not found", &ticker);
            return Err(Box::new(e))
        },
    }
}

fn main() {
    // let r = get_quote("AAPL", "1d").unwrap();
    // println!("{}", r);
    let matches = App::new("Stonks")
        .version("0.1.0")
        .author("Zach Peters")
        .about("Look up stocks and things")
        .arg(Arg::with_name("stocks")
            .required(true)
            .help("the stock you want")
            .index(1)
        )
        .get_matches();

    // TODO what format should the date be?
    // TODO should stocks be a list?
    let stocks = matches.value_of("stocks").unwrap_or("APPL");
    let q = get_quote(&stocks, "1d").unwrap();
    println!("Stock: {}", q);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_basic_quote() {
        let ticker = "AAPL";
        let period = "1h";
        let result = get_quote(ticker, period).unwrap();
        assert!(result.contains("AAPL"));
    }
}
