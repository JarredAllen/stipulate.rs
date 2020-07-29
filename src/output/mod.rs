#[cfg(feature = "print-output")]
mod cli;

// type ClassResults = HashMap<String, StudentResults>;
// type StudentResults = HashMap<String, Result<TestAnswer, Box<dyn Error + 'static>>>;
use super::ClassResults;

pub trait OutputMode {
    fn output_class_results(
        &mut self,
        results: &ClassResults,
    ) -> Result<(), Box<dyn std::error::Error + 'static>>;
}

pub fn get_output_mode(name: &str) -> Option<Box<dyn OutputMode + 'static>> {
    match name {
        #[cfg(feature = "print-output")]
        "print" => Some(Box::new(cli::Print::<std::io::Stdout>::with_stdout())),
        _ => None,
    }
}
