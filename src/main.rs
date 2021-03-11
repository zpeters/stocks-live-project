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

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    //return two price differences: one
    // as a percentage of the starting price,
    // one as an absolute difference between the
    //first and the last price of the period.
    if !series.is_empty() {
        let first_price = series.first().unwrap();
        let last_price = series.last().unwrap();
        let abs_diff = last_price - first_price;
        let percent_diff = abs_diff / first_price * 100.0;
        Some((percent_diff, abs_diff))
    } else {
        None
    }
}

fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && n > 1 {
        Some(
            series
                .windows(n)
                .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                .collect(),
        )
    } else {
        None
    }
}

fn max(series: &[f64]) -> Option<f64> {
    let mut max_found: Option<f64> = None;
    for s in series.iter() {
        if max_found.is_none() {
            max_found = Some(*s);
        }
        if *s > max_found.unwrap() {
            max_found = Some(*s);
        }
    }
    max_found
}

fn min(series: &[f64]) -> Option<f64> {
    let mut min_found: Option<f64> = None;
    for s in series.iter() {
        if min_found.is_none() {
            min_found = Some(*s);
        }
        if *s < min_found.unwrap() {
            min_found = Some(*s);
        }
    }
    min_found
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
    fn test_min_aggregate() {
        let expected: f64 = 0.00001;
        let input: &[f64] = &[0.123, 0.012, 0.00001];
        let result = min(&input).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_aggregate() {
        let expected: f64 = 0.123;
        let input: &[f64] = &[0.123, 0.012, 0.00001];
        let result = max(&input).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_n_window_sma() {
        let input_usize: usize = 3;
        let input_series: &[f64] = &[1.0, 2.0, 3.5, 4.5, 12.2];
        let expected = Some(vec![
            2.1666666666666665,
            3.3333333333333335,
            6.733333333333333,
        ]);

        let actual = n_window_sma(input_usize, input_series);

        assert_eq!(actual, expected);
    }

    // TODO implement
    #[test]
    fn test_price_diff() {
        let input_series: &[f64] = &[1.0, 2.0, 3.5, 4.5, 12.2];
        let expected_percentage = 1120.0;
        let expected_absolute = 11.2;
        let expected = Some((expected_percentage, expected_absolute));

        let result = price_diff(input_series);

        assert_eq!(result, expected)
    }
}
