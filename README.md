# Workflow

OK 1. Create a Rust binary application project with cargo.

OK 2. Ingest stock quote data from an API.
        The crate yahoo_finance_api provides a great starting point. Focus on the crate’s blocking API for now and specify the blocking feature on import.
        Try different intervals and periods for the API, you want the highest possible resolution for your custom aggregations. What are its limits?
        Using Rust’s Option and Result types, how can you handle API and connection errors without panicking?

OK 3. Use command-line parameters to pass in the stock symbols and “from” date.
        The Rust standard library offers a way to access command line arguments, but …
        can a third-party crate make your life easier?

4. Calculate performance indicators for the given period.
        A period is the time between the “from” date and the current date
        Aggregate the closing (adjclose) prices and find their minimum (fn min(series: &[f64]) -> Option<f64>) and maximum (fn max(series: &[f64]) -> Option<f64>) across the period. What data structures and types from the standard library can you use?
        Calculate a simple moving average over the entire series. Here is the recommended function interface: fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>>, where the series parameter is a std::slice with one value per day.
        Using the function interface fn price_diff(series: &[f64]) -> Option<(f64, f64)>, return two price differences: one as a percentage of the starting price, one as an absolute difference between the first and the last price of the period.

5. The company’s data pipeline expects a CSV file for input, so you decide to print the results in that format to stdout:
        Display numbers (the min/max prices, change, and 30-day-average) with at most two decimal places
        Use stderr to communicate any errors
        The following columns are important to the company:
            The date of the last quote retrieved
            The stock symbol
            The close price for the last quote
            The change since the beginning (close) price of the period, expressed in percentage of that price
            The period’s minimum price
            The period’s maximum price
            The last 30-day-average
        Here is an example output:

    period start,symbol,price,change %,min,max,30d avg
    2020-07-02T19:30:00+00:00,MSFT,$206.25,50.42%,$131.65,$207.85,$202.35
    2020-07-02T19:30:00+00:00,AAPL,$364.12,79.07%,$192.88,$371.38,$363.40
    2020-07-02T19:30:00+00:00,UBER,$30.68,-30.39%,$14.39,$44.73,$30.38

6. Test with the following stock symbols: MSFT, GOOG, AAPL, UBER,IBM.


