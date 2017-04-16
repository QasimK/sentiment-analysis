extern crate clap;
extern crate hyper;
extern crate hyper_native_tls;
extern crate select;
extern crate itertools;

use std::collections::{BTreeMap, HashSet};
use std::io::Read;

use clap::{App, Arg};
use hyper::{Client, Url};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use select::document::Document;
use select::predicate::{Attr, Class};
use itertools::Itertools;


fn main() {
    let matches = App::new("sentiment")
        .version("0.1")
        .about("Determine the sentiment of a given text/URL.")
        .author("QasimK")
        .arg(Arg::with_name("INPUT")
            .help("Sets the text or the URL of the web page to analyse")
            .required(true)
            .index(1))
        .arg(Arg::with_name("v")
            .short("v")
            .help("Sets the level of verbosity"))
        .get_matches();

    let verbosity = matches.occurrences_of("v");
    let input = matches.value_of("INPUT").unwrap();
    let text = if input.starts_with("http") {
        if verbosity >= 1 {
            println!("Downloading web page...");
        }
        // TODO: Horrible mixture of as_str, to_string, as_str etc. etc. here
        slice_html(input, download_src(input).as_str())
    } else {
        input.to_string()
    };

    println!("{:?}", analyse(text.as_str()));
}


fn download_src(url: &str) -> String {
    // TODO: Better error handling
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(e) => panic!("Invalid URL\n{:?}", e),
    };

    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let mut response = match client.get(url).send() {
        Ok(response) => response,
        Err(err) => panic!("Failed to read response\n{:?}", err),
    };

    let mut buffer = String::new();
    match response.read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(e) => panic!("Failure to download:\n{:?}", e),
    }
}


// TODO: Return &str - problem with lifetimes/borrowing when trying to return node.text()
fn slice_html(url: &str, page: &str) -> String {
    // Return the article text from the full html page
    let ordered_web_selectors = include_str!("data/web_selectors.txt");
    for line in ordered_web_selectors.lines() {
        // TODO: Get the word and score directly without all this vector-collection business
        let v: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(v.len(), 2);
        let domain = v[0];
        let pattern = v[1];
        if url.contains(domain) {
            println!("Detected {:?}, selecting {:?}...", domain, pattern);
            let document = Document::from(page);
            let (selector_type, core_pattern) = (&pattern[0..1], &pattern[1..]);
            // TODO: Somehow combine these two branches despite different Attr vs Class
            match selector_type {
                "#" => {
                    println!("ID matching...");
                    for node in document.find(Attr("id", core_pattern)) {
                        // println!("{:?}", node.text());
                        return node.text();
                    }
                }
                "." => {
                    println!("Class matching...");
                    for node in document.find(Class(core_pattern)) {
                        // println!("{:?}", node.text());
                        return node.text();
                    }
                }
                _ => panic!("Unsupported selector {:?}", pattern),
            };
        }
    }

    String::new()
}

fn analyse(text: &str) -> i32 {
    // Read the word-to-sentiment-score library
    let ordered_word_scores = include_str!("data/afinn/AFINN-en-165.txt");

    // Uniqueness by Chars (vs Grapheme clusters) should be fine here...
    let valid_chars: HashSet<char> = ordered_word_scores.chars().unique().collect();

    // TODO: 31 words that have a space in them
    let mut word_to_score = BTreeMap::new();
    for line in ordered_word_scores.lines() {
        let v: Vec<&str> = line.splitn(2, "\t").collect();
        let word = v[0];
        let score = v[1].parse::<i32>().unwrap();
        word_to_score.insert(word, score);
    }

    // Compute the score
    println!("Scoring Words...");
    let scores: Vec<i32> = text
        .to_lowercase()  // Known words and chars are in lower-case
        .chars()
        .filter(|char| valid_chars.contains(char))  // Remove unknown characters
        .collect::<String>()
        .split_whitespace()
        .filter(|word| word_to_score.contains_key(word))  // Filter out for better avg
        .map(|word| match word_to_score.get(word) {
                Some(&score)    => {
                    println!("++ {:?} {:?}", word, score);
                    score
                },
                // Superfluous match due to above filter
                None            => {
                    // println!("-- {:?} {:?}", word, 0);
                    0
                },
            })
        .collect();

    let sum: i32 = scores.iter().sum();
    let len = scores.len();
    let avg = sum as f32 / len as f32;
    println!("Sum: {:?}, Len: {:?}", sum, len);

    // Word scores are between -5 and 5, so multiple to give -100 to 100 rating
    (avg * 20.0) as i32
}
