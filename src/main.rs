mod fetch;
pub mod helpers;
pub mod io;
pub mod parser;
pub mod utils;

use crate::{
    fetch::{get_quarterly_urls, get_quarterly_xmls, get_ticker_map, get_ticker_submission},
    io::save::Output,
    parser::xml::XBRLFiling,
    utils::{get_bar, print_loading},
};
use clap::Parser;
use indicatif::ProgressIterator;

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

// Configurations

const SAVE_PATH: &str = "output";
// Can be "json", "facts", "dimensions"
const OUTPUT: [&'static str; 1] = ["json"];

const TOTAL_STEPS: i32 = 4;

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
    let current = 0;
    let current = print_loading("Getting submissions", current, TOTAL_STEPS);
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
    let current = print_loading("Getting quarterly urls", current, TOTAL_STEPS);
    let urls = get_quarterly_urls(&submission);

    // Get quarterly quarterly raw xml files
    let current = print_loading("Getting raw xml files", current, TOTAL_STEPS);
    let raw_xmls = get_quarterly_xmls(&urls);

    assert_eq!(urls.len(), raw_xmls.len());

    // parse xml
    let current = print_loading("Parsing xml files", current, TOTAL_STEPS);
    let mut filings: Vec<XBRLFiling> = Vec::new();
    for i in (0..urls.len()).progress_with(get_bar(urls.len())) {
        filings.push(XBRLFiling::new(
            urls[i].clone(),
            raw_xmls[i].clone(),
            OUTPUT.to_vec(),
        ))
    }

    // save to files
    print_loading("Saving to files", current, TOTAL_STEPS);
    let save_path = format!("{}/{}", SAVE_PATH, args.ticker);

    for filing in filings.iter().progress_with(get_bar(filings.len())) {
        if OUTPUT.contains(&"json") {
            let data = Output::Json(filing.json.clone().expect("No json"));
            let file_name = format!("{}.json", filing.info.xml_name.clone()).to_string();
            data.save(save_path.clone(), file_name);
        }

        if OUTPUT.contains(&"facts") {
            let data = Output::Facts(filing.facts.clone().expect("No facts"));
            let file_name = format!("facts_{}.csv", filing.info.xml_name.clone()).to_string();
            data.save(save_path.clone(), file_name);
        }

        if OUTPUT.contains(&"dimensions") {
            let data = Output::Dimensions(filing.dimensions.clone().expect("No dimensions"));
            let file_name = format!("dimensions_{}.csv", filing.info.xml_name.clone()).to_string();
            data.save(save_path.clone(), file_name);
        }
    }
}
