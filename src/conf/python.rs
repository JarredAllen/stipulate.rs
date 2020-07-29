use std::collections::HashMap;
use std::time::Duration;

use errormake::errormake;

/// Default timeout for python programs, in seconds, per test case
const DEFAULT_TIMEOUT: u64 = 5;

/// The default python interpreter to use, if unspecified
#[cfg(target_family = "windows")]
const DEFAULT_PYTHON: &str = "python";
#[cfg(any(target_family = "unix"))]
const DEFAULT_PYTHON: &str = "python3";

/// This struct represents a configuration for running a python program.
///
/// See `PythonConfig::from_toml` for docs on how to create one.
pub struct PythonConfig {
    name: String,
    test_data_dir: String,
    python_version: String,
    timeout: Option<Duration>,
    filename: String,
    args: Vec<String>,
    target_dir: String,
}

impl PythonConfig {
    /// Required fields in the toml:
    ///  - "name": A name for this test
    ///  - "tests_dir": The directory to contain input and output data
    ///  - "file": The file to be run
    ///
    /// Optional fields in the toml:
    ///  - "timeout": Should be the number of seconds to allow before
    /// timing out, `true` (use default timeout value), or `false`
    /// (allow tested code to run however long it takes - not
    /// recommended). Default: 5 seconds
    ///  - "args": Should be an array of arguments to pass to the python
    /// program being tested. It will be passed to the sys.argv value
    /// in the python program. Default: empty array
    ///  - "version": Enables you to specify a version of python to use.
    /// Default: OS dependent: "python" for Windows, "python3" for
    /// Linux/MacOS.
    pub fn from_toml(
        conf: &toml::Value,
    ) -> Result<PythonConfig, PythonConfigError<std::convert::Infallible>> {
        let name = match conf.get("name") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(PythonConfigError::with_description(
                "Missing \"name\" field".to_string(),
            )),
            _ => Err(PythonConfigError::with_description(
                "\"name\" field should be a string".to_string(),
            )),
        }?;
        let test_data_dir = match conf.get("tests_dir") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(PythonConfigError::with_description(
                "Missing \"tests_dir\" field".to_string(),
            )),
            _ => Err(PythonConfigError::with_description(
                "\"tests_dir\" field should be a string".to_string(),
            )),
        }?;
        let python_version = match conf.get("version") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Ok(String::from(DEFAULT_PYTHON)),
            _ => Err(PythonConfigError::with_description(
                "\"version\", if specified, must be a string".to_string(),
            )),
        }?;
        let timeout = match conf.get("timeout") {
            Some(toml::Value::Integer(seconds)) => Ok(Some(Duration::new(*seconds as u64, 0))),
            Some(toml::Value::Float(seconds)) => Ok(Some(Duration::new(
                *seconds as u64,
                ((seconds % 1.0) * 1e9) as u32,
            ))),
            None | Some(toml::Value::Boolean(true)) => Ok(Some(Duration::new(DEFAULT_TIMEOUT, 0))),
            Some(toml::Value::Boolean(false)) => Ok(None),
            _ => Err(PythonConfigError::with_description(
                "\"timeout\", if specified, should be a number or false".to_string(),
            )),
        }?;
        let filename = match conf.get("file") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(PythonConfigError::with_description(
                "Missing \"file\" field".to_string(),
            )),
            _ => Err(PythonConfigError::with_description(
                "\"file\" field should be a string".to_string(),
            )),
        }?;
        let args: Vec<String> = match conf.get("args") {
            None => Ok(Vec::new()),
            Some(toml::Value::Array(arr)) => arr
                .iter()
                .map(|v| match v {
                    toml::Value::String(s) => Ok(s.clone()),
                    toml::Value::Array(_) | toml::Value::Table(_) => {
                        Err(PythonConfigError::with_description(
                            "Args may not contain nested structures".to_string(),
                        ))
                    }
                    toml::Value::Integer(i) => Ok(format!("{}", i)),
                    toml::Value::Float(f) => Ok(format!("{}", f)),
                    toml::Value::Boolean(b) => Ok(format!("{}", b)),
                    toml::Value::Datetime(d) => Ok(format!("{}", d)),
                })
                .collect(),
            _ => Err(PythonConfigError::with_description(
                "\"args\", if specified, must be an array".to_string(),
            )),
        }?;
        let target_dir = match conf.get("target_dir") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(PythonConfigError::with_description(
                "Missing \"target_dir\" field".to_string(),
            )),
            _ => Err(PythonConfigError::with_description(
                "\"target_dir\" field must be a string".to_string(),
            )),
        }?;
        Ok(PythonConfig {
            name,
            test_data_dir,
            python_version,
            timeout,
            filename,
            args,
            target_dir,
        })
    }
}

impl super::Config for PythonConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn test_type(&self) -> super::TestType {
        super::TestType::Directory(&self.test_data_dir)
    }

    fn case_timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    fn command(&self, _student_dir: &str) -> String {
        String::from(&self.python_version)
    }

    fn args(&self, student_dir: &str) -> Vec<String> {
        // In this block, we pretend that args_refs was actually just
        // the Vec<&str> that the borrow checker doesn't let it be.
        let mut args = vec![format!("{}/{}", student_dir, self.filename)];
        args.extend(self.args.iter().cloned());
        args
    }

    fn do_setup(&self, _student_dir: &str) -> bool {
        // No setup needs to be done
        true
    }

    fn target_dir(&self) -> &str {
        &self.target_dir
    }

    fn env_vars(&self, _student_dir: &str) -> HashMap<String, String> {
        // No work needs to be done
        HashMap::new()
    }
}

errormake!(#[doc="An error while interpreting Python configuration"] pub PythonConfigError);
