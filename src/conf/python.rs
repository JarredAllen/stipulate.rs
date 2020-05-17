use std::time::Duration;

use toml;

const FOO: [String; 0] = [];

pub struct PythonConfig {
}

impl PythonConfig {
    pub fn from_toml(conf: &toml::Value) -> Result<PythonConfig, &'static str> {
        Err("Python not implemented yet")
    }
}

impl super::Config for PythonConfig {
    /// A name for this set of tests
    fn name(&self) -> &str {
        "foo"
    }

    /// The folder containing the input and output files
    fn test_data_dir(&self) -> &str {
        "foo"
    }

    /// The amount of time to let code run before timing out
    fn case_timeout(&self) -> &Option<Duration> {
        &None
    }

    /// The name of the command to run.
    fn command(&self) -> &str {
        "foo"
    }

    /// The arguments to be passed to the command.
    fn args(&self) -> &[String] {
        &FOO[..]
    }

    /// A list of commands to be run in the student's code directory
    /// before running the code.
    fn setup(&self) -> &[String] {
        &FOO[..]
    }
}
