use std::time::Duration;
use std::env;
use std::thread;

use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;
use pest::Parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct InputParser;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Error, Debug)]
enum WaitError {
    #[error("Invalid arguments")]
    InvalidArgs,
    #[error("There was a problem parsing the input")]
    CannotParse(ParseError)
}

impl From<ParseError> for WaitError {
    fn from(value: ParseError) -> WaitError {
        WaitError::CannotParse(value)
    }
}

struct Time(Duration);

#[derive(Error, Debug)]
enum ParseError {
    #[error("Cannot understand split {0}")]
    CannotParse(String)
}

impl TryFrom<String> for Time {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lists = match InputParser::parse(Rule::list, &value) {
            Ok(l) => l,
            Err(_) => return Err(ParseError::CannotParse(value)),
        };

        let mut duration = Duration::default();

        for list in lists {
            for section in list.into_inner() {
                let mut inner_rules = section.into_inner();

                // For a rule match we know there's going to be two fields.
                let count: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();
                let multi: u64 = match inner_rules.next().unwrap().as_str() {
                    "s" => 1,
                    "m" => 60,
                    "h" => 60 * 60,
                    _   => unreachable!() // Technically not possible.
                };

                duration += Duration::from_secs(count * multi);
            }
        }

        Ok(Time(duration))
    }
}

fn main() {
    match go() {
        Ok(_) => (),
        Err(e) => eprintln!("Can't continue, {}", e)
    }
}

fn go() -> Result<(), WaitError> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: {} <time to wait>", args[0]);
        println!("  time to wait can be 5m for 5 minutes, 12s for 12 seconds, etc.");
        return Err(WaitError::InvalidArgs);
    }

    let to_wait = Time::try_from(args[1].clone())?.0;
    let mut waited = Duration::default();

    println!("Waiting {:#?}", to_wait);

    let pb = ProgressBar::new(to_wait.as_secs());
    pb.set_style(ProgressStyle::default_bar()
        .template("💤 [{elapsed_precise}] [{wide_bar:.cyan/blue}] [{eta_precise}]")
        .progress_chars("#>-"));

    while waited < to_wait {
        pb.set_position(waited.as_secs());
        waited += REFRESH_INTERVAL;
        thread::sleep(REFRESH_INTERVAL);
    }

    Ok(())
}
