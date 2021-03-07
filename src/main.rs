use chrono::DateTime;
use clap::{values_t, App, Arg};
use yahoo_finance_api as yahoo;

// TODO parse stock to some kind of struct, prices in a vect?
// TODO get max and min (see 4.b.)

fn get_quote(ticker: &str, period: &str) -> Result<String, String> {
    let provider = yahoo::YahooConnector::new();

    match provider.get_quote_range(ticker, "1m", period) {
        Ok(response) => match response.quotes() {
            Ok(quotes) => Ok(format!(
                "{} quotes for {} are: {:?}",
                ticker, period, quotes
            )),
            Err(e) => Err(format!("Unknown error: {:?}", e)),
        },
        Err(e) => Err(format!("Ticker '{}' not found. {:?}", &ticker, e)),
    }
}

fn from_to_date(from_date: &str) -> Result<String, String> {
    match DateTime::parse_from_rfc3339(&from_date) {
        Ok(parsed) => {
            // how many days ago is this (diff)
            // return formatted string 'Nd' for 'N' days
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
            Ok(q) => println!("Stock: {}", q),
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
        assert!(result.contains("AAPL"), "result should contain stock name");
        dbg!(&result);
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
}
