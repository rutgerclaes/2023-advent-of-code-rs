use float_next_after::NextAfter;
use utils::prelude::*;
use itertools::Itertools;

fn main() {
    setup_logging();

    let input: Vec<String> = read_input_lines().expect( "Input could not be read" );
    let (times,distances) = parse_input( &input ).expect( "Input could not be parsed" );
    
    let part_one = part_one( &times, &distances );
    show_part_one( part_one );
    
    let part_two = part_two( &times, &distances );
    show_part_one( part_two );
}

fn part_one( times: &[u32], distances: &[u32] ) -> u64 {
    times.iter().zip( distances.iter() ).map( |(&time,&distance)| { let (a,b) = calculate_range( time as u64, distance as u64); b - a + 1 } ).product()
}

fn part_two( times: &[u32], distances: &[u32] ) -> u64 {
    let total_time = times.iter().fold( 0, |total,time| *time as u64 + total * 10u64.pow( (*time as f32).log10().ceil() as u32 ) );
    let total_distance = distances.iter().fold( 0, |total,distance| *distance as u64 + total * 10u64.pow( (*distance as f32).log10().ceil() as u32 ) );

    let (a,b) = calculate_range( total_time, total_distance );
    b - a + 1
}

fn parse_input( lines: &[String] ) -> SolutionResult<(Vec<u32>,Vec<u32>)> {
    let (times, distances) = lines.iter().map( |line| line.split_ascii_whitespace().skip(1).map( |d| d.parse() ).try_collect() ).collect_tuple().ok_or_else( || SolutionError::InputParsingFailed( "Could not extract exactl y 2 lines".to_owned()) )?;
    Ok( (times?, distances?) )
}

fn calculate_range( total_time: u64, distance: u64 ) -> (u64,u64) {
    let t: f64 = total_time as f64;
    let d: f64 = distance as f64;

    let d1 = ( t - ( t * t - 4f64 * d ).sqrt() ) / 2f64;
    let d2 = ( t + ( t * t - 4f64 * d ).sqrt() ) / 2f64;

    let d1 =  d1.next_after( std::f64::MAX ).ceil() as u64;
    let d2 =  d2.next_after( std::f64::MIN ).floor() as u64;

    (d1,d2)
}

#[cfg(test)]
mod test {
    use utils::owned;

    use crate::{calculate_range, parse_input};

    #[test]
    fn test_range_calculation() {
        assert_eq!( (2,5), calculate_range( 7, 9) );
        assert_eq!( (4,11), calculate_range( 15, 40) );
        assert_eq!( (11,19), calculate_range( 30, 200) );
    }

    #[test]
    fn test_input_parsing() {
        let input = vec![ owned!("Time:      7  15   30"), owned!("Distance:  9  40  200") ];
        let (times,distances) = parse_input( &input ).expect( "Parsing input failed" );

        assert_eq!( vec![7, 15, 30], times );
        assert_eq!( vec![9, 40, 200], distances );
    }
}