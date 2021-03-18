use anyhow::Result;
use async_std;
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

async fn get_quote(ticker: &str, period: &str) -> Result<Stock> {
    let provider = yahoo::YahooConnector::new();
    let response = provider.get_quote_range(ticker, "1m", period).await?;
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

fn generate_report_line(start_date: &str, ticker: &str, series: &[f64]) -> String {
    let precision_dollars = 2;
    let precision_percent = 3;
    let last_close_price = series.last().unwrap();
    let (percent_diff, _abs_diff) = price_diff(&series).unwrap();
    let min = min(&series).unwrap();
    let max = max(&series).unwrap();
    let thirty_day_window = n_window_sma(30, &series).unwrap();
    let last_thirty_day_window = thirty_day_window.last().unwrap();

    format!(
        "{2},{3},${4:.0$},{5:.1$}%,${6:.0$},${7:.0$},${8:.0$}",
        precision_dollars,
        precision_percent,
        start_date,
        ticker,
        last_close_price,
        percent_diff,
        min,
        max,
        last_thirty_day_window
    )
}

async fn print_report(stocks: Vec<String>, from: String) {
    let date = from_to_date(&from).unwrap();

    println!("period start,symbol,price,change %,min,max,30d avg");
    for stock in &stocks {
        match get_quote(&stock, &date).await {
            Ok(q) => {
                let close_series: &[f64] = &q.quotes.iter().map(|x| x.close).collect::<Vec<f64>>();
                let report_line = generate_report_line(&from, &stock, &close_series);
                println!("{}", report_line);
            }
            Err(e) => println!("Could not retrieve stock information. Error {:?}", e),
        }
    }
}

#[async_std::main]
async fn main() -> () {
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

    let from = matches.value_of("from").unwrap().to_string();
    let stocks = values_t!(matches.values_of("stocks"), String).unwrap_or_else(|e| e.exit());

    print_report(stocks, from).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_get_basic_quote() {
        let ticker = "AAPL";
        let period = "1h";
        let result = get_quote(ticker, period).await.unwrap();
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

    #[test]
    fn test_price_diff() {
        let input_series: &[f64] = &[1.0, 2.0, 3.5, 4.5, 12.2];
        let expected_percentage = 1120.0;
        let expected_absolute = 11.2;
        let expected = Some((expected_percentage, expected_absolute));

        let result = price_diff(input_series);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_generate_report_line() {
        let input_ticker = "XYZ";
        let input_date = "2020-07-02T19:30:00+00:00";
        let input_series: &[f64] = &[
            1.0, 2.0, 3.5, 4.5, 12.2, 1.0, 2.0, 3.5, 4.5, 12.2, 1.0, 2.0, 3.5, 4.5, 12.2, 1.0, 2.0,
            3.5, 4.5, 12.2, 1.0, 2.0, 3.5, 4.5, 12.2, 1.0, 2.0, 3.5, 4.5, 12.2, 1.0, 2.0, 3.5, 4.5,
            12.2,
        ];

        let expected = "2020-07-02T19:30:00+00:00,XYZ,$12.20,1120.000%,$1.00,$12.20,$4.64";

        let actual = generate_report_line(&input_date, &input_ticker, &input_series);

        assert_eq!(expected, actual);
    }
}
