use regex::{Captures, Regex};

use crate::prelude::{SolutionError, SolutionResult};

pub fn capture_regex<'a>(regex: &Regex, input: &'a str) -> SolutionResult<Captures<'a>> {
    regex
        .captures(input)
        .ok_or_else(|| SolutionError::no_regex_match(regex, input))
}

pub fn named_match<'a>(captures: &Captures<'a>, name: &str) -> SolutionResult<&'a str> {
    captures
        .name(name)
        .ok_or_else(|| SolutionError::no_regex_capture(name.to_owned()))
        .map(|s| s.as_str())
}
