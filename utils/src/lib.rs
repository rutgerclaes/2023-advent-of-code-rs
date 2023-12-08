pub mod io;
pub mod result;

pub mod prelude {

    pub use crate::io::input::{parse_input_lines, read_input};
    pub use crate::io::output::{setup_logging, show_part_one, show_part_two};
    pub use crate::result::{SolutionError, SolutionResult};
}
