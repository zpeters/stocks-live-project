use anyhow::Result;
use chrono::DateTime;
use clap::{values_t, App, Arg};
use yahoo_finance_api as yahoo;

// TODO parse stock to some kind of struct, prices in a vect?
// TODO get max and min (see 4.b.)

#[derive(Debug)]
struct Stock {
    ticker: String,
    quotes: Vec<yahoo::Quote>,
}

impl Stock {
    fn min(&self) -> Option<f64> {
        let mut min_found: Option<f64> = None;
        for q in self.quotes.iter() {
            if min_found.is_none() {
                min_found = Some(q.adjclose);
            }
            if q.adjclose < min_found.unwrap() {
                min_found = Some(q.adjclose);
            }
        }
        min_found
    }
    fn max(&self) -> Option<f64> {
        let mut max_found: Option<f64> = None;
        for q in self.quotes.iter() {
            if max_found.is_none() {
                max_found = Some(q.adjclose);
            }
            if q.adjclose > max_found.unwrap() {
                max_found = Some(q.adjclose);
            }
        }
        max_found
    }
}

fn get_quote(ticker: &str, period: &str) -> Result<Stock> {
    let provider = yahoo::YahooConnector::new();
    let response = provider.get_quote_range(ticker, "1m", period)?;
    let quotes = response.quotes()?;
    let stock = Stock {
        ticker: String::from(ticker),
        quotes,
    };
    Ok(stock)
}

fn from_to_date(from_date: &str) -> Result<String, String> {
    match DateTime::parse_from_rfc3339(&from_date) {
        Ok(parsed) => {
            let now = chrono::Local::now();
            let now_fixed = now.with_timezone(now.offset());
            let diff = now_fixed - parsed;
            let num_days = diff.num_days();
            Ok(format!("{}d", num_days))
        }
        Err(e) => Err(format!("Date parsing error: {:?}", e)),
    }
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
                .min_values(1)
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

    let from = matches.value_of("from").unwrap();
    let stocks = values_t!(matches.values_of("stocks"), String).unwrap_or_else(|e| e.exit());

    let date = from_to_date(&from).unwrap();

    for stock in &stocks {
        match get_quote(&stock, &date) {
            Ok(q) => println!("Stock: {:?}", q),
            Err(e) => println!("Could not retrieve stock information. Error {:?}", e),
        }
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
        assert_eq!(result.ticker, "AAPL", "result should contain stock name");
        assert_eq!(result.quotes.len(), 60, "we should have exactly 60 results")
    }

    #[test]
    fn test_from_to_date_bad_date() {
        let input = "fjkdsa;fjkd";
        let result = from_to_date(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_to_date() {
        let input = "2020-01-01T00:00:00Z";
        let result = from_to_date(&input).unwrap();
        assert!(result.contains("d"), "result should contain 'd' for 'days'")
    }
    #[test]
    fn test_from_to_date_future() {
        let input = "3020-01-01T00:00:00Z";
        let result = from_to_date(&input).unwrap();
        assert!(
            result.contains("-"),
            "result should contain '-' since it is in the future"
        );
        assert!(result.contains("d"), "result should contain 'd' for 'days'");
    }

    #[test]
    fn test_max_aggregate() {
        let expected: f64 = 0.123;
        let input = Stock {
            ticker: String::from("MyFakeStock"),
            quotes: vec![
                yahoo::Quote {
                    timestamp: 0000000,
                    open: 1.0,
                    high: 1.0,
                    low: 1.0,
                    volume: 1,
                    close: 1.0,
                    adjclose: 0.123,
                },
                yahoo::Quote {
                    timestamp: 0000001,
                    open: 22.1234,
                    high: 184.00,
                    low: 0.01,
                    volume: 1,
                    close: 1.0,
                    adjclose: 0.012,
                },
                yahoo::Quote {
                    timestamp: 0000002,
                    open: 1.0,
                    high: 1.0,
                    low: 3.0,
                    volume: 1,
                    close: 1.0,
                    adjclose: 0.00001,
                },
            ],
        };

        let result = input.max().unwrap();

        assert_eq!(result, expected);
    }
}
