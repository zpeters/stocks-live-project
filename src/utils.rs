use chrono::prelude::*;
use yahoo_finance_api as yahoo;

pub use yahoo::Quote as TickerQuote;
pub use yahoo::YahooError as DataSourceError;

pub async fn fetch_ticker_data(
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    interval: String,
) -> std::result::Result<Vec<TickerQuote>, DataSourceError> {
    let api = yahoo::YahooConnector::new();
    let ticker_data = api.get_quote_history_interval(&symbol, from, to, &interval);

    match ticker_data.await {
        Ok(response) => response.quotes(),
        Err(e) => Err(e),
    }
}

pub async fn price_diff(a: &[f64]) -> Option<(f64, f64)> {
    if !a.is_empty() {
        let (first, last) = (a.first().unwrap(), a.last().unwrap());
        let abs_diff = last - first;
        let first = if *first == 0.0 { 1.0 } else { *first };
        let rel_diff = abs_diff / first;
        Some((abs_diff, rel_diff))
    } else {
        None
    }
}

pub async fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
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

pub async fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
    }
}

pub async fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_price_diff() -> std::io::Result<()> {
        assert_eq!(price_diff(&[]).await, None);
        assert_eq!(price_diff(&[1.0]).await, Some((0.0, 0.0)));
        assert_eq!(price_diff(&[1.0, 0.0]).await, Some((-1.0, -1.0)));
        assert_eq!(
            price_diff(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await,
            Some((8.0, 4.0))
        );
        assert_eq!(
            price_diff(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some((1.0, 1.0))
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_min() -> std::io::Result<()> {
        assert_eq!(min(&[]).await, None);
        assert_eq!(min(&[1.0]).await, Some(1.0));
        assert_eq!(min(&[1.0, 0.0]).await, Some(0.0));
        assert_eq!(min(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await, Some(1.0));
        assert_eq!(min(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await, Some(0.0));
        Ok(())
    }

    #[async_std::test]
    async fn test_max() -> std::io::Result<()> {
        assert_eq!(max(&[]).await, None);
        assert_eq!(max(&[1.0]).await, Some(1.0));
        assert_eq!(max(&[1.0, 0.0]).await, Some(1.0));
        assert_eq!(max(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await, Some(10.0));
        assert_eq!(max(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await, Some(6.0));
        Ok(())
    }

    #[async_std::test]
    async fn test_n_window_sma() -> std::io::Result<()> {
        let series = vec![2.0, 4.5, 5.3, 6.5, 4.7];

        assert_eq!(
            n_window_sma(3, &series).await,
            Some(vec![3.9333333333333336, 5.433333333333334, 5.5])
        );
        assert_eq!(n_window_sma(5, &series).await, Some(vec![4.6]));
        assert_eq!(n_window_sma(10, &series).await, Some(vec![]));
        Ok(())
    }
}
