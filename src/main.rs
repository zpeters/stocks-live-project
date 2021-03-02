use chrono::DateTime;
use clap::{App, Arg};
use std::error::Error;
use yahoo_finance_api as yahoo;

fn get_quote(ticker: &str, period: &str) -> Result<String, Box<dyn Error>> {
    let provider = yahoo::YahooConnector::new();

    match provider.get_quote_range(ticker, "1m", period) {
        Ok(response) => match response.quotes() {
            Ok(quotes) => Ok(format!(
                "{} quotes for {} are: {:?}",
                ticker, period, quotes
            )),
            Err(e) => {
                eprintln!("Inner error");
                return Err(Box::new(e));
            }
        },
        Err(e) => {
            eprintln!("Ticker '{}' not found", &ticker);
            return Err(Box::new(e));
        }
    }
}

// TODO implement
// see from_date_validator
// use rfc3339
// https://rust-lang-nursery.github.io/rust-cookbook/datetime/parse.html
fn from_to_date(_from_date: &str) -> String {
    todo!();
}

fn from_date_validator(datetime: String) -> Result<(), String> {
    match DateTime::parse_from_rfc3339(&datetime) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Date parsing error: {:?}", e)),
    }
}

fn main() {
    let matches = App::new("Stonks")
        .version("0.1.0")
        .author("Zach Peters")
        .about("Look up stocks and things")
        .arg(
            Arg::with_name("stocks")
                .required(true)
                .help("the stock you want")
                .index(1),
        )
        .arg(
            Arg::with_name("from")
                .help("From date to start getting info from.  Date format is in RFC3339 format.")
                .short("f")
                .required(true)
                .long("from")
                .validator(from_date_validator)
                .takes_value(true),
        )
        .get_matches();

    // TODO stocks should get a list
    // TODO calculate date to # days
    // TODO give date example
    // validate date
    let from = matches.value_of("from").unwrap();
    let stocks = matches.value_of("stocks").unwrap_or("APPL");

    let date = from_to_date(&from);

    match get_quote(&stocks, &date) {
        Ok(q) => println!("Stock: {}", q),
        Err(e) => println!("Could not retrieve stock information. Error {:?}", e),
    }
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
