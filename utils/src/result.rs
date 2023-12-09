use std::{io, num::ParseIntError};

use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("Could not parse problem input: {0}")]
    InputParsingFailed(String),

    #[error("No solution was found")]
    NoSolutionFound,
}

impl SolutionError {
    pub fn no_regex_match(regex: &Regex, input: &str) -> SolutionError {
        Self::InputParsingFailed(format!(
            "Regular expression {} failed to match on input '{}'",
            regex, input
        ))
    }

    pub fn no_regex_capture(name: String) -> SolutionError {
        Self::InputParsingFailed(format!("Could not get named match '{name}'"))
    }
}

impl From<ParseIntError> for SolutionError {
    fn from(value: ParseIntError) -> Self {
        Self::InputParsingFailed(format!("Parsing of an integer failed: {}", value))
    }
}

impl From<io::Error> for SolutionError {
    fn from(value: io::Error) -> Self {
        Self::InputParsingFailed(format!("Reading input failed: {}", value))
    }
}

pub type SolutionResult<T> = Result<T, SolutionError>;
