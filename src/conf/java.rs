use std::time::Duration;

use toml;

const DEFAULT_TIMEOUT: u64 = 5;

pub struct JavaConfig {
    name: String,
    test_data_dir: String,
    timeout: Option<Duration>,
    args: Vec<String>,
}

impl JavaConfig {
    pub fn from_toml(conf: &toml::Value) -> Result<JavaConfig, &'static str> {
        let name = match conf.get("name") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err("Missing \"name\" field"),
            _ => Err("\"name\" field should be a string"),
        }?;
        let test_data_dir = match conf.get("tests_dir") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err("Missing \"tests_dir\" field"),
            _ => Err("\"tests_dir\" field should be a string"),
        }?;
        let main_class = match conf.get("main_class") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err("Missing \"main_class\" field"),
            _ => Err("\"main_class\" field should be a string"),
        }?;
        let timeout = match conf.get("timeout") {
            Some(toml::Value::Integer(seconds)) => Ok(Some(Duration::new(*seconds as u64, 0))),
            Some(toml::Value::Float(seconds)) => Ok(Some(Duration::new(*seconds as u64, ((seconds % 1.0) * 1e9) as u32))),
            None | Some(toml::Value::Boolean(true)) => Ok(Some(Duration::new(DEFAULT_TIMEOUT, 0))),
            Some(toml::Value::Boolean(false)) => Ok(None),
            _ => Err("\"timeout\", if specified, should be a number or false"),
        }?;
        let args = match conf.get("args") {
            None => Ok(vec![main_class]),
            Some(toml::Value::Array(_arr)) => Err("Custom arguments to main not yet supported"),
            _ => Err("\"args\", if specified, must be an array"),
        }?;
        Ok(JavaConfig { name, test_data_dir, timeout, args })
    }
}

impl super::Config for JavaConfig {
    /// A name for this set of tests
    fn name(&self) -> &str {
        &self.name
    }

    /// The folder containing the input and output files
    fn test_data_dir(&self) -> &str {
        &self.test_data_dir
    }

    /// The amount of time to let code run before timing out
    fn case_timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    /// The name of the command to run.
    fn command(&self) -> &str {
        "java"
    }

    /// The arguments to be passed to the command.
    fn args(&self) -> &[String] {
        self.args.as_slice()
    }

    /// A list of commands to be run in the student's code directory
    /// before running the code.
    fn setup(&self) -> &[String] {
        self.args.as_slice()
    }
}
