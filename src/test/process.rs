use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

/// Reads from an input stream until the input stream ends, and returns
/// the results in a `String`.
fn read_from_stream<T: Read>(stream: &mut T) -> Result<String, String> {
    let mut data = Vec::new();
    stream
        .read_to_end(&mut data)
        .map_err(|e| format!("{}", e))?;
    String::from_utf8(data).map_err(|e| format!("{}", e))
}

/// An enum which contains the possible results of running a Test on a
/// student's code. Note that this only has options for if the test
/// completes, a different value is returned if the tester is unable to
/// evaluate a student's code.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TestAnswer {
    /// It passed the test
    Success,
    /// It failed the test, and no additional information is given
    Failure,
    /// It did not finish running during the allotted time.
    Timeout,
    /// It failed the test. This contains a `String` with more
    /// information, which can be given to the student.
    FailWithMessage(String),
}

/// Runs the given command with the given args, and passes the given
/// argument as input through standard input. It returns true iff the
/// command's output matches `expected_output`.
///
/// If timeout is None, then it will wait for the child to finish.
/// Otherwise, it will only wait the specified amount of time.
///
/// It returns true if it matches, false if it doesn't match, and Err
/// if it encountered an error trying to evaluate it (with an &str
/// explaining the error caused).
///
/// For now, it assumes that the child process sends valid UTF-8 out.
/// If it doesn't, then this function will error.
pub fn test_output(
    cmd: &str,
    args: &[&str],
    input: &str,
    expected_output: &str,
    timeout: Option<Duration>,
) -> Result<TestAnswer, String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Error spawning child to test: {}", e))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| String::from("Error grabbing child stdin"))?
        .write_all(input.as_bytes())
        .map_err(|e| format!("Error writing to child stdin: {}", e))?;
    match timeout {
        Some(delay) => match child.wait_timeout(delay) {
            Ok(Some(code)) => Ok(code),
            Ok(None) => {
                let _ = child.kill();
                if let Err(e) = child.wait() {
                    println!("Error reaping child process: {}", e);
                };
                return Ok(TestAnswer::Timeout);
            }
            Err(e) => Err(e),
        },
        None => child.wait(),
    }
    .map_err(|e| format!("Error waiting on child process: {}", e))?;
    let child_output = read_from_stream(
        child
            .stdout
            .as_mut()
            .ok_or_else(|| String::from("Error grabbing child stdout"))?,
    )?;
    Ok(match child_output == expected_output {
        true => TestAnswer::Success,
        false => TestAnswer::Failure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_timeout() {
        assert_eq!(
            test_output("echo", &["Hello, world"], "", "Hello, world\n", None),
            Ok(TestAnswer::Success)
        );
        assert_eq!(
            test_output("echo", &["Goodbye, world"], "", "Hello, world\n", None),
            Ok(TestAnswer::Failure)
        );
    }

    #[test]
    fn test_with_timeout() {
        assert_eq!(
            test_output(
                "echo",
                &["Hello, world"],
                "",
                "Hello, world\n",
                Some(Duration::new(1, 0))
            ),
            Ok(TestAnswer::Success)
        );
        assert_eq!(
            test_output(
                "echo",
                &["Goodbye, world"],
                "",
                "Hello, world\n",
                Some(Duration::new(1, 0))
            ),
            Ok(TestAnswer::Failure)
        );
        assert_eq!(
            test_output(
                "sleep",
                &["10"],
                "",
                "Hello, world\n",
                Some(Duration::new(0, 100))
            ),
            Ok(TestAnswer::Timeout)
        );
    }
}
