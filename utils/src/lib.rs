pub mod io;
pub mod parsing;
pub mod result;

#[macro_export]
macro_rules! owned {
    ($s:expr) => {
        $s.to_owned()
    };
}

pub mod prelude {

    pub use crate::owned;

    pub use crate::io::input::{parse_input_lines, read_input, read_input_lines};
    pub use crate::io::output::{
        setup_logging, show_part_one, show_part_two, show_result_part_one, show_result_part_two,
    };
    pub use crate::result::{SolutionError, SolutionResult};

    pub use crate::parsing::{capture_regex, named_match};
}
