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

/// Tests the given command (cmd and args) against the given cases
/// (input/ouput pairs), with a specified per-case timeout.
///
/// It returns a vector containing the results of testing on each of the
/// cases, in the order given.
fn test_student_against_strings(
    student_dir: &str,
    setup: &[&str],
    cmd: &str,
    args: &[&str],
    cases: &[(&str, &str)],
    timeout: Option<Duration>,
) -> Vec<Result<TestAnswer, Box<dyn Error + 'static>>> {
    // TODO move to student_dir and do the setup commands
    if true {
        cases
            .iter()
            .map(|(input, output)| test_output_against_strings(cmd, args, input, output, timeout))
            .collect()
    } else {
        vec![TestAnswer::CompileError; cases.len()]
            .into_iter()
            .map(|c| Ok(c))
            .collect()
    }
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
) -> Result<
    HashMap<String, HashMap<String, Result<TestAnswer, Box<dyn Error + 'static>>>>,
    Box<dyn Error + 'static>,
> {
    lazy_static! {
        static ref FILENAME_EXT_REMOVER: Regex = Regex::new(r"(.*)[.][^.]+").unwrap();
    }
    match config.test_type() {
        TestType::Directory(dir) => {
            let cases: Vec<String> = fs::read_dir(dir)?
                .map(|file| {
                    file.map(|f| {
                        String::from(
                            f.file_name()
                                .to_str()
                                .expect("Error parsing filename as unicode"),
                        )
                    })
                })
                .collect::<Result<Vec<String>, _>>()?
                .iter()
                .filter_map(|fname| {
                    FILENAME_EXT_REMOVER
                        .captures(fname)
                        .map(|caps| caps.get(1))
                        .flatten()
                })
                .map(|name| String::from(name.as_str()))
                .unique()
                .collect();
            let inputs: Vec<String> = cases
                .iter()
                .map(|case| {
                    let mut in_data = String::new();
                    File::open(format!("{}/{}.in", dir, case))
                        .map_err(|e| format!("{:?}", e))?
                        .read_to_string(&mut in_data)
                        .map_err(|e| format!("{:?}", e))?;
                    Ok(in_data)
                })
                .collect::<Result<Vec<String>, String>>()?;
            let outputs: Vec<String> = cases
                .iter()
                .map(|case| {
                    let mut out_data = String::new();
                    File::open(format!("{}/{}.out", dir, case))
                        .map_err(|e| format!("{:?}", e))?
                        .read_to_string(&mut out_data)
                        .map_err(|e| format!("{:?}", e))?;
                    Ok(out_data)
                })
                .collect::<Result<Vec<String>, String>>()?;
            let test_data: Vec<(&str, &str)> = inputs
                .iter()
                .map(|s| s.as_str())
                .zip(outputs.iter().map(|s| s.as_str()))
                .collect();
            fs::read_dir(config.target_dir())
                .map_err(|e| format!("{:?}", e))?
                .map(|student_dir| {
                    let student_dir = student_dir.map_err(|e| format!("{:?}", e))?;
                    let student_path = student_dir.path();
                    let student_name = String::from(
                        student_dir
                            .file_name()
                            .to_str()
                            .expect("Error parsing student folder name as utf-8"),
                    );
                    let test_results = test_student_against_strings(
                        student_path.to_str().unwrap(),
                        config.setup(),
                        config.command(),
                        config.args(),
                        test_data.as_slice(),
                        *config.case_timeout(),
                    );
                    Ok((
                        student_name,
                        cases
                            .iter()
                            .cloned()
                            .zip(test_results.into_iter())
                            .collect(),
                    ))
                })
                .collect()
        }
    }
}
