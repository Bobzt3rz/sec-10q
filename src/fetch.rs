use std::collections::HashMap;

use indicatif::ProgressIterator;
use reqwest::{blocking::Client, header::USER_AGENT};
use serde::Deserialize;
use serde_json::{Number, Value};

use crate::utils::{get_bar, without_dash};

const LIST_JSON_URL: &str = "https://www.sec.gov/files/company_tickers.json";

// format: https://data.sec.gov/submissions/CIK<10_digit_cik>.json
const BASE_SUBMISSION_URL: &str = "https://data.sec.gov/submissions";
// format: https://www.sec.gov/Archives/edgar/data/<cik>/<accession_number_no_dash>/<lowercase_ticker>-<report_date_no_dash>_htm.xml
const BASE_QUARTERLY_URL: &str = "https://www.sec.gov/Archives/edgar/data";
const ua: &str = "bunnavit bunnavit.sa@gmail.com";

#[derive(Deserialize, Debug)]
struct JSONResponseInner {
    cik_str: Number,
    ticker: String,
    title: String,
}
#[derive(Debug)]
pub struct TickerMapContent {
    pub cik: Number,
    pub title: String,
}

pub fn get_ticker_map() -> HashMap<String, TickerMapContent> {
    let client = Client::new();
    let response = client
        .get(LIST_JSON_URL)
        .header(USER_AGENT, ua)
        .send()
        .expect("Failed to send request");
    let json_map: HashMap<String, JSONResponseInner> = match response.json() {
        Ok(json) => json,
        Err(e) => panic!("Error: {}", e),
    };
    let mut ticker_hashmap = HashMap::new();
    for entry in json_map {
        ticker_hashmap.insert(
            entry.1.ticker,
            TickerMapContent {
                cik: entry.1.cik_str,
                title: entry.1.title,
            },
        );
    }
    ticker_hashmap
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntrySubmission {
    pub filings: Filings,
    pub addresses: Value,
    pub category: String,
    pub cik: String,
    pub entity_type: String,
    pub sic: String,
    pub sic_description: String,
    pub insider_transaction_for_owner_exists: Number,
    pub insider_transaction_for_issuer_exists: Number,
    pub description: String,
    pub ein: String,
    pub website: String,
    pub investor_website: String,
    pub state_of_incorporation: String,
    pub state_of_incorporation_description: String,
    pub phone: String,
    pub flags: String,
    pub former_names: Value,
    pub fiscal_year_end: Value,
    pub exchanges: Vec<String>,
    pub tickers: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Filings {
    pub files: Vec<Files>,
    pub recent: Submission,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    pub name: String,
    pub filing_count: Number,
    pub filing_from: String,
    pub filing_to: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub accession_number: Vec<String>,
    pub filing_date: Vec<String>,
    pub report_date: Vec<String>,
    pub acceptance_date_time: Vec<String>,
    pub act: Vec<String>,
    pub form: Vec<String>,
    pub file_number: Vec<String>,
    pub film_number: Vec<String>,
    pub items: Vec<String>,
    pub size: Vec<Number>,
    pub is_XBRL: Vec<Number>,
    pub is_inline_XBRL: Vec<Number>,
    pub primary_document: Vec<String>,
    pub primary_doc_description: Vec<String>,
}

pub fn get_ticker_submission(ticker_content: &TickerMapContent) -> EntrySubmission {
    let mut cik_url = ticker_content.cik.to_string();
    while cik_url.len() < 10 {
        cik_url = format!("{}{}", 0, cik_url);
    }
    // format: https://data.sec.gov/submissions/CIK<10_digit_cik>.json
    let full_url = format!("{}/CIK{}.json", BASE_SUBMISSION_URL, cik_url);
    println!("{:?}", full_url);

    let client = Client::new();
    let response = client
        .get(full_url)
        .header(USER_AGENT, ua)
        .send()
        .expect("Failed to send request");
    let output: EntrySubmission = match response.json() {
        Ok(json) => json,
        Err(e) => panic!("Error: {}", e),
    };
    // println!("{:?}", output.tickers);
    output
}

fn get_old_submission(file_name: &String) -> Submission {
    let full_url = format!("{}/{}", BASE_SUBMISSION_URL, file_name);
    let client = Client::new();
    let response = client
        .get(full_url)
        .header(USER_AGENT, ua)
        .send()
        .expect("Failed to send request");
    let output: Submission = match response.json() {
        Ok(json) => json,
        Err(e) => panic!("Error: {}", e),
    };
    output
}

pub fn get_quarterly_urls(submission: &EntrySubmission) -> Vec<String> {
    // format: https://www.sec.gov/Archives/edgar/data/<cik>/<accession_number_no_dash>/<lowercase_ticker>-<report_date_no_dash>_htm.xml
    // format: https://www.sec.gov/Archives/edgar/data/<cik>/<accession_number_no_dash>/<primary_document>

    // NOTE: there is XML file but it's not the primary file  => Have to check for this case.
    // format: pltr-20210331.xml
    let mut url_array = Vec::new();
    let mut submissions = Vec::new();
    let ticker = &submission.tickers[0];
    let cik = &submission.cik;
    submissions.push(submission.filings.recent.clone());
    // get old filings if there are any
    for file in submission
        .filings
        .files
        .iter()
        .progress_with(get_bar(submission.filings.files.len()))
    {
        let old_submission = get_old_submission(&file.name);
        submissions.push(old_submission.clone());
        println!("fetched submission: {}", submissions.len());
    }

    println!("submissions: {:#?}", submissions.len());
    // loop through every submission ever sent
    for submission in submissions.iter().progress_with(get_bar(submissions.len())) {
        // match only quarterlies in each submission
        for (i, entry) in submission.form.iter().enumerate() {
            match entry.as_str() {
                "10-Q" => {
                    let is_inline_xbrl =
                        submission.is_inline_XBRL[i].to_string() == String::from("1");
                    let file_name = match is_inline_xbrl {
                        true => format!(
                            "{}_htm.xml",
                            submission.primary_document[i]
                                .split('.')
                                .collect::<Vec<_>>()[0]
                        ),
                        // automated xml generated by SEC if main document is not inline_xbrl
                        false => format!(
                            "{}-{}.xml",
                            ticker.to_lowercase(),
                            without_dash(&submission.report_date[i])
                        ),
                    };
                    url_array.push(format!(
                        "{}/{}/{}/{}",
                        BASE_QUARTERLY_URL,
                        cik,
                        without_dash(&submission.accession_number[i]),
                        file_name,
                    ));
                }
                _ => (),
            }
        }
    }
    println!("files obtained: {}", url_array.len());
    url_array
}

pub fn get_quarterly_xmls(urls: &Vec<String>) -> Vec<String> {
    let mut raw_xmls = Vec::new();
    let client = Client::new();
    for url in urls.iter().progress_with(get_bar(urls.len())) {
        let response = client
            .get(url)
            .header(USER_AGENT, ua)
            .send()
            .expect("Failed to send request");
        let raw_xml: String = match response.text() {
            Ok(text) => text,
            Err(e) => panic!("get_quarterly_xmls Error: {}", e),
        };
        raw_xmls.push(raw_xml);
    }
    raw_xmls
}
