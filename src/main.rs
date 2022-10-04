mod fetch;
pub mod helpers;
pub mod parser;
pub mod utils;

use crate::{
    fetch::{get_quarterly_urls, get_quarterly_xmls, get_ticker_map, get_ticker_submission},
    parser::xml::XBRLFiling,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ticker of the company to query
    #[arg(short, long, default_value = "no-input")]
    ticker: String,

    /// List ticker of all available companies
    #[arg(short, long)]
    list: bool,
}

fn main() {
    let mut args = Args::parse();

    if args.list {
        println!("Finding list of available tickers...");
        let ticker_map = get_ticker_map();
        for entry in ticker_map {
            println!("{}: {}", entry.0, entry.1.title)
        }
        return;
    }

    if args.ticker == "no-input" {
        println!("Please enter a company ticker using cargo run --name <company-ticker>");
        return;
    }
    // Find ticker info
    args.ticker = args.ticker.to_uppercase();
    let ticker_map = get_ticker_map();
    let ticker_info = match ticker_map.get(&args.ticker) {
        Some(info) => info,
        None => panic!("{} is not found in the registry!", args.ticker),
    };
    println!("{}", args.ticker);
    println!(
        "You queried for {}: {}!",
        ticker_info.title, ticker_info.cik
    );

    // Get ticker submissions
    let submission = get_ticker_submission(ticker_info);

    // Get quarterly urls
    let urls = get_quarterly_urls(&submission);

    // Get quarterly quarterly raw xml files
    let raw_xmls = get_quarterly_xmls(&urls);

    // let output = ["json", "facts", "dimensions"].to_vec();
    let output = ["json"].to_vec();

    let filing = XBRLFiling::new(urls[0].clone(), raw_xmls[0].clone(), output);

    println!("{:#?}", filing.json);
}
