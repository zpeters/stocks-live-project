use yahoo_finance_api as yahoo;
use std::error::Error;

fn get_quote(ticker: &str, period: &str) -> Result<String, Box<dyn Error>> {
    let provider = yahoo::YahooConnector::new();

    match provider.get_quote_range(ticker, "1m", period) {
        Ok(response) => {
            match response.quotes() {
                Ok(quotes) => {
                    Ok(format!("{} quotes for {} are: {:?}", ticker, period, quotes))
                },
                Err(e) => return Err(Box::new(e)),
            }
        },
        Err(e) => return Err(Box::new(e)),
    }
}

fn main() {
    let r = get_quote("AAPL", "1d").unwrap();
    println!("{}", r);
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
