//! Functions, enumerations, etc. pertaining to the evaluation of student programs

mod process;

use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::time::Duration;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

use super::conf::{TestConfig, TestType};
use process::test_output_against_strings;
pub use process::TestAnswer;

/// A struct representing a single test case for a directory test. It
/// contains an input and an output.
pub struct TestCase {
    input: String,
    output: String,
}
impl TestCase {
    /// Returns the input string
    fn get_input(&self) -> &String {
        &self.input
    }

    /// Returns the output string
    fn get_output(&self) -> &String {
        &self.output
    }
}
/// A HashMap mapping test case names to the result of running on that test case
pub type StudentResults = HashMap<String, Result<TestAnswer, Box<dyn Error + 'static>>>;
/// A HashMap mapping student names to their results
pub type ClassResults = HashMap<String, StudentResults>;

/// Tests the given command (cmd and args) against the given cases
/// (input/ouput pairs), with a specified per-case timeout.
///
/// It returns a vector containing the results of testing on each of the
/// cases, in the order given.
///
/// This method assumes that the necessary setup has been done already
fn test_student_against_test_case(
    cmd: String,
    args: Vec<String>,
    env_vars: &HashMap<String, String>,
    cases: &HashMap<String, TestCase>,
    timeout: Option<Duration>,
) -> StudentResults {
    cases
        .iter()
        .map(|(case_name, case_data)| {
            (
                case_name.clone(),
                test_output_against_strings(
                    &cmd,
                    &args,
                    &env_vars,
                    case_data.get_input(),
                    case_data.get_output(),
                    timeout,
                ),
            )
        })
        .collect()
}

/// Runs a test given the configuration, for all students in the
/// directory given by the configuration.
///
/// If there's an issue loading the folder specified by the config, then
/// it will return the relevant error. Otherwise, it will return a
/// HashMap mapping student names to a hash map mapping test names to
/// that student's results on that test
pub fn test_from_configuration(
    config: &TestConfig,
) -> Result<ClassResults, Box<dyn Error + 'static>> {
    lazy_static! {
        static ref FILENAME_EXT_REMOVER: Regex = Regex::new(r"(.*)[.][^.]+").unwrap();
    }
    match config.test_type() {
        TestType::Directory(dir) => {
            let cases: Vec<String> = fs::read_dir(dir)?
                .filter_map(|file| {
                    match file.map(|f| {
                        String::from(
                            f.file_name()
                                .to_str()
                                .expect("Error parsing filename as unicode"),
                        )
                    }) {
                        Ok(filename) => Some(String::from(
                            FILENAME_EXT_REMOVER
                                .captures(&filename)
                                .map(|caps| caps.get(1))
                                .flatten()?
                                .as_str(),
                        )),
                        Err(_) => None,
                    }
                })
                .unique()
                .collect();
            let inputs: Vec<String> = cases
                .iter()
                .map(|case| {
                    let mut in_data = String::new();
                    File::open(format!("{}/{}.in", dir, case))?.read_to_string(&mut in_data)?;
                    Ok(in_data)
                })
                .collect::<Result<Vec<_>, Box<dyn Error + 'static>>>()?;
            let outputs: Vec<String> = cases
                .iter()
                .map(|case| {
                    let mut out_data = String::new();
                    File::open(format!("{}/{}.out", dir, case))?.read_to_string(&mut out_data)?;
                    Ok(out_data)
                })
                .collect::<Result<Vec<_>, Box<dyn Error + 'static>>>()?;
            let test_data: HashMap<String, TestCase> = cases
                .into_iter()
                .zip(
                    inputs
                        .into_iter()
                        .zip(outputs.into_iter())
                        .map(|(input, output)| TestCase { input, output }),
                )
                .collect();
            fs::read_dir(config.target_dir())?
                .filter_map(|entry| {
                    // Remove directories and file i/o errors
                    let entry = entry.ok()?;
                    match entry.file_type() {
                        Ok(filetype) => {
                            if filetype.is_dir() {
                                Some(entry)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                })
                .map(|student_dir| {
                    // Now, let's test the students
                    let student_path = student_dir.path();
                    let student_path = student_path.to_str().expect("Error loading student folder");
                    let student_name = String::from(
                        student_dir
                            .file_name()
                            .to_str()
                            .expect("Error parsing student folder name as utf-8"),
                    );
                    if !config.do_setup(student_path) {
                        return Ok((
                            student_name,
                            test_data
                                .keys()
                                .map(|k| (k.clone(), Ok(TestAnswer::CompileError)))
                                .collect(),
                        ));
                    }
                    let env_vars = config.env_vars(student_path);
                    let test_results = test_student_against_test_case(
                        config.command(student_path),
                        config.args(student_path),
                        &env_vars,
                        &test_data,
                        *config.case_timeout(),
                    );
                    Ok((student_name, test_results))
                })
                .collect()
        }
    }
}
