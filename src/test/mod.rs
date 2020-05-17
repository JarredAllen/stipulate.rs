//! Functions, enumerations, etc. pertaining to the evaluation of student programs

mod process;

use std::time::Duration;

pub use process::test_output;
pub use process::TestAnswer;

/// Tests the given command (cmd and args) against the given cases
/// (input/ouput pairs), with a specified per-case timeout.
///
/// It returns a vector containing the results of testing on each of the
/// cases, in the order given.
pub fn test_student<T>(
    cmd: &str,
    args: &[&str],
    cases: &[(&str, &str)],
    timeout: Option<Duration>,
) -> Vec<Result<TestAnswer, String>> {
    cases
        .iter()
        .map(|(input, output)| test_output(cmd, args, input, output, timeout))
        .collect()
}
