mod csv;
#[cfg(feature = "table-output")]
mod table;

use std::fs::File;

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
        #[cfg(feature = "table-output")]
        "print" => Some(Box::new(table::Table::with_stdout())),
        "csv" => Some(Box::new(csv::CsvOutput::with_stdout())),
        _ => None,
    }
}

pub fn get_output_mode_for_file(
    name: &str,
    filename: &str,
) -> Option<Box<dyn OutputMode + 'static>> {
    let file = File::create(filename).ok()?;
    match name {
        #[cfg(feature = "table-output")]
        "print" => Some(Box::new(table::Table::with_output(file))),
        "csv" => Some(Box::new(csv::CsvOutput::with_output(file))),
        _ => None,
    }
}
