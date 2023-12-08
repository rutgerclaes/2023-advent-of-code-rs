use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("Could not parse problem input: {0}")]
    InputParsingFailed(String),

    #[error("No solution was found")]
    NoSolutionFound,
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
