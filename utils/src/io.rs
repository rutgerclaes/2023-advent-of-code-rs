pub mod input {
    use std::env;
    use std::fs::File;
    use std::io::{self, stdin, BufReader, Read};
    pub fn read_input() -> io::Result<BufReader<Box<dyn Read>>> {
        if let Some(path) = env::args().nth(1) {
            tracing::debug!(file = path, "reading input");
            let file = File::open(path)?;
            Ok(BufReader::new(Box::new(file) as Box<dyn Read>))
        } else {
            tracing::debug!("reading input from stdin");
            let stdin = stdin();
            Ok(BufReader::new(Box::new(stdin.lock()) as Box<dyn Read>))
        }
    }
}

pub mod output {
    use std::fmt::Display;

    use ansi_term::{Color::Green, Style};
    use tracing_subscriber::{filter::LevelFilter, EnvFilter};

    pub fn setup_logging() {
        let filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::ERROR.into())
            .from_env_lossy();
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .pretty()
            .with_env_filter(filter)
            .init();
    }

    pub fn show<T: Display>(part: &str, value: T) {
        println!(
            "Solution to {}: {}",
            Style::new().bold().paint(part),
            Green.bold().paint(format!("{value}"))
        );
    }

    pub fn show_part_one<T: Display>(value: T) {
        show("part 1", value)
    }

    pub fn show_part_two<T: Display>(value: T) {
        show("part 2", value)
    }
}
