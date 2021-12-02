use std::time::Duration;
use std::env;
use std::thread;

use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Error, Debug)]
enum WaitError {
    #[error("Invalid arguments")]
    InvalidArgs,
    #[error("There was a problem parsing")]
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
    #[error("Cannot understand suffix {0}")]
    UnrecognisedSuffix(char),
    #[error("Cannot understand split {0}")]
    CannotParse(String)
}

impl TryFrom<String> for Time {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut result = Duration::default();
        let splits: Vec<&str> = value.split_inclusive(|c: char| c.is_alphabetic()).collect();

        for split in splits {
            let nums = &split[..split.len()-1].parse::<u64>();
            let denomination = &split.chars().last();

            match (nums, denomination) {
                (Ok(num), Some(c)) => match c {
                    's' => result += Duration::from_secs(*num),
                    'm' => result += Duration::from_secs(num * 60),
                    'h' => result += Duration::from_secs(num * 60 * 60),
                    _ => return Err(ParseError::UnrecognisedSuffix(*c))
                },
                _ => return Err(ParseError::CannotParse(split.to_string()))
                // TODO: Should properly report what was the error rather than a simple CannotParse...
            }
        }

        Ok(Time(result))
    }
}

fn main() -> Result<(), WaitError> {
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
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}]")
        .progress_chars("#>-"));

    while waited < to_wait {
        pb.set_position(waited.as_secs());
        waited += REFRESH_INTERVAL;
        thread::sleep(REFRESH_INTERVAL);
    }

    Ok(())
}
