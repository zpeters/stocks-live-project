# Readme

## Notes

this is from Mannings "Building a Stock-Tracking CLI With Async Streams in Rust" Live Project

## TODO - Milestone 2


1.    **OK** Write tests to make sure evolving the code won’t break it:
        Your min and max functions.
        The simple moving average.
        For calculating the relative and absolute differences over the entire period.

2.    **OK** Transform your current code to be asynchronous:
        Don’t forget to make your tests async too
        Testing async functions works just as if they were running in the main function
        Mixing async libraries may lead to strange errors and incompatibilities. Stick to something like async-std, tokio
        yahoo_finance_api's default API is asnyc starting with version 0.3. Remove the blocking feature from your Cargo.toml to use it.

3.    Use actors to do the actual data processing:
        Wrap your code to work in actors
        Find ways to publish and subscribe to messages without explicit calls
        There are several actor frameworks available - pick one that fits your async library; for example Xactor with async-std or Actix and tokio. Note: the solution will be based on Xactor.

4.    Continuously fetch stock quotes for each symbol:
        It’s critical to fetch the data for every 30 second interval
        Don’t sleep the thread, multiple tasks are running there; find an actor-based alternative
        Use one or more actors to run the previous data processing algorithms (min, max, 30 day simple moving average, price change).

5.    The CTO liked the previous console output format, so you decide to keep it:

    period start,symbol,price,change %,min,max,30d avg
    2020-07-02T19:30:00+00:00,MSFT,$206.25,50.42%,$131.65,$207.85,$202.35
    2020-07-02T19:30:00+00:00,AAPL,$364.12,79.07%,$192.88,$371.38,$363.40
    2020-07-02T19:30:00+00:00,UBER,$30.68,-30.39%,$14.39,$44.73,$30.38

6.    Polish your code.
        Can you write more tests to document function limitations and usage?
        Are you missing data points because of an increasing backlog?
        Are there other ways to implement this data processing pipeline?

7.    Run test with many symbols, like those contained in the S&P 500 index.
        You can download a list of the S&P 500 May 2020 symbols here:
