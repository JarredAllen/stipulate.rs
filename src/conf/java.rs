use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;

use errormake::errormake;

use glob::glob;

/// Default timeout for java programs, in seconds, per test case
const DEFAULT_TIMEOUT: u64 = 5;

/// This struct represents a configuration for running a java program.
///
/// See `JavaConfig::from_toml` for docs on how to create one.
pub struct JavaConfig {
    name: String,
    test_data_dir: String,
    timeout: Option<Duration>,
    main_class: String,
    args: Vec<String>,
    target_dir: String,
}

impl JavaConfig {
    /// Required fields in the toml:
    ///  - "name": A name for this test
    ///  - "tests_dir": The directory to contain input and output data
    ///  - "main_class": The class containing a public static void
    ///  - "target_dir": The directory containing all student
    ///    submissions (each submission as its own directory).
    /// main(String[] args) method to be run.
    ///
    /// Optional fields in the toml:
    ///  - "timeout": Should be the number of seconds to allow before
    /// timing out, `true` (use default timeout value), or `false`
    /// (allow tested code to run however long it takes - not
    /// recommended). Default: 5 seconds
    ///  - "args": Should be an array of arguments to pass to the java
    /// program being tested. It will be passed directly to the String[]
    /// args in the java program. Default: empty array.
    pub fn from_toml(
        conf: &toml::Value,
    ) -> Result<JavaConfig, JavaConfigError<std::convert::Infallible>> {
        let name = match conf.get("name") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(JavaConfigError::with_description(
                "Missing \"name\" field".to_string(),
            )),
            _ => Err(JavaConfigError::with_description(
                "\"name\" field should be a string".to_string(),
            )),
        }?;
        let test_data_dir = match conf.get("tests_dir") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(JavaConfigError::with_description(
                "Missing \"tests_dir\" field".to_string(),
            )),
            _ => Err(JavaConfigError::with_description(
                "\"tests_dir\" field should be a string".to_string(),
            )),
        }?;
        let main_class = match conf.get("main_class") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(JavaConfigError::with_description(
                "Missing \"main_class\" field".to_string(),
            )),
            _ => Err(JavaConfigError::with_description(
                "\"main_class\" field should be a string".to_string(),
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
            _ => Err(JavaConfigError::with_description(
                "\"timeout\", if specified, should be a number or boolean".to_string(),
            )),
        }?;
        let args: Vec<String> = match conf.get("args") {
            None => Ok(Vec::new()),
            Some(toml::Value::Array(arr)) => arr
                .iter()
                .map(|v| match v {
                    toml::Value::String(s) => Ok(s.clone()),
                    toml::Value::Array(_) | toml::Value::Table(_) => {
                        Err(JavaConfigError::with_description(
                            "Args may not contain nested structures".to_string(),
                        ))
                    }
                    toml::Value::Integer(i) => Ok(format!("{}", i)),
                    toml::Value::Float(f) => Ok(format!("{}", f)),
                    toml::Value::Boolean(b) => Ok(format!("{}", b)),
                    toml::Value::Datetime(d) => Ok(format!("{}", d)),
                })
                .collect(),
            _ => Err(JavaConfigError::with_description(
                "\"args\", if specified, must be an array".to_string(),
            )),
        }?;
        let target_dir = match conf.get("target_dir") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err(JavaConfigError::with_description(
                "Missing \"target_dir\" field".to_string(),
            )),
            _ => Err(JavaConfigError::with_description(
                "\"target_dir\" field must be a string".to_string(),
            )),
        }?;
        Ok(JavaConfig {
            name,
            test_data_dir,
            timeout,
            main_class,
            args,
            target_dir,
        })
    }
}

impl super::Config for JavaConfig {
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
        String::from("java")
    }

    fn args(&self, _student_dir: &str) -> Vec<String> {
        let mut args = self.args.clone();
        args.insert(0, self.main_class.clone());
        args
    }

    fn do_setup(&self, student_dir: &str) -> bool {
        let source_glob = format!("{}/*.java", student_dir);
        let source_files: Vec<std::path::PathBuf> = match match glob(&source_glob) {
            Ok(files) => files,
            Err(_) => return false,
        }
        .collect()
        {
            Ok(files) => files,
            Err(_) => return false,
        };
        Command::new("javac")
            .args(source_files)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .map_or(false, |mut child| {
                child.wait().map_or(false, |signal| signal.success())
            })
    }

    fn target_dir(&self) -> &str {
        &self.target_dir
    }

    fn env_vars(&self, student_dir: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(String::from("CLASSPATH"), String::from(student_dir));
        vars
    }
}

errormake!(#[doc="An error while interpreting Java configuration"] pub JavaConfigError);
