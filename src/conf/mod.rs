//! Handles loading of configurations for tests

mod java;
mod python;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

pub use java::JavaConfig;
pub use python::PythonConfig;

/// This struct represents all of the configuration for a test run.
///
/// It is essentially a smart pointer to an object of type `Config`,
/// with some extra convenience methods about using it.
pub struct TestConfig {
    config: Box<dyn Config>,
}
impl TestConfig {
    /// Returns a reference to the config contained in here
    pub fn get_config(&self) -> &dyn Config {
        self.config.as_ref()
    }

    /// Returns a mutable reference to the config contained in here
    pub fn get_config_mut(&mut self) -> &mut dyn Config {
        self.config.as_mut()
    }

    /// Loads a given filename into a configuration
    ///
    /// See `TestConfig::from_toml_values` for information about what it
    /// can do.
    pub fn from_file(filename: &str) -> Result<TestConfig, Box<dyn Error + 'static>> {
        let mut file = File::open(filename)?;
        let file_contents: toml::Value = read_from_stream(&mut file)?.parse()?;
        Self::from_toml_values(file_contents)
    }

    /// Loads the configuration from the given parsed toml.
    ///
    /// All keys and section headers should be lower-case (and it is
    /// case-sensitive).
    ///
    /// The file should have one section header, whose name is the kind
    /// of test being run. The available options currently are "java"
    /// and "python".
    ///
    /// Configuration options for java are at `JavaConfig::from_toml`.
    ///
    /// Configuration options for python are at `PythonConfig::from_toml`.
    pub fn from_toml_values(values: toml::Value) -> Result<TestConfig, Box<dyn Error + 'static>> {
        match values {
            toml::Value::Table(table) => {
                if table.len() == 1 {
                    let key = table.keys().find(|_| true).unwrap();
                    let value = table.get(key).unwrap();
                    Ok(TestConfig {
                        config: match key.as_str() {
                            "java" => Box::new(java::JavaConfig::from_toml(value)?),
                            "python" => Box::new(python::PythonConfig::from_toml(value)?),
                            key => Err(Box::new(InterpretConfigError::new(format!(
                                "Unrecognized config type: {}",
                                key
                            ))))?,
                        },
                    })
                } else {
                    Err(Box::new(InterpretConfigError::new(String::from(
                        "The config file should have exactly one section",
                    ))))
                }
            }
            _ => Err(Box::new(InterpretConfigError::new(String::from(
                "The config file wasn't a table (shouldn't be thrown)",
            )))),
        }
    }
}

impl Deref for TestConfig {
    type Target = dyn Config;

    fn deref(&self) -> &Self::Target {
        self.config.as_ref()
    }
}
impl DerefMut for TestConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.config.as_mut()
    }
}

/// The trait implemented by all supported configurations.
pub trait Config {
    /// A name for this set of tests
    fn name(&self) -> &str;

    /// The kind of test to run (see `TestType` for options)
    fn test_type(&self) -> TestType;

    /// The amount of time to let code run before timing out
    fn case_timeout(&self) -> &Option<Duration>;

    /// The name of the command to run.
    fn command(&self) -> &str;

    /// The arguments to be passed to the command.
    fn args(&self) -> &[&str];

    /// A list of commands to be run in the student's code directory
    /// before running the code.
    fn setup(&self) -> &[&str];

    fn target_dir(&self) -> &str;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InterpretConfigError {
    message: String,
}
impl InterpretConfigError {
    pub fn new(message: String) -> InterpretConfigError {
        InterpretConfigError { message }
    }
}
impl std::fmt::Display for InterpretConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for InterpretConfigError {}

/// The different kinds of tests that can be done.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestType<'a> {
    /// Load in testing data from a directory.
    ///
    /// For each test case, there should be a file <test_case_name>.in
    /// and another file <test_case_name>.out, which contain,
    /// respectively, the input and output for that test case.
    Directory(&'a str),
}

/// Reads from an input stream until the input stream ends, and returns
/// the results in a `String`, decoded as UTF8.
fn read_from_stream<T: Read>(stream: &mut T) -> Result<String, Box<dyn Error + 'static>> {
    let mut data = Vec::new();
    stream.read_to_end(&mut data)?;
    Ok(String::from_utf8(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_toml() {
        let java_toml: toml::Value =
            "[java]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nmain_class = \"Main\"\ntarget_dir = \"testa/sub\"\n"
                .parse()
                .unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test A", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["Main"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&Some(Duration::new(5, 0)), java_config.case_timeout());
        assert_eq!("testa/sub", java_config.target_dir());
        let java_toml: toml::Value = "[java]\nname = \"Test B\"\ntests_dir = \"path/to/test\"\nmain_class = \"MainB\"\ntimeout = 1\ntarget_dir = \"testb/sub\"\n".parse().unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test B", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["MainB"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&Some(Duration::new(1, 0)), java_config.case_timeout());
        assert_eq!("testb/sub", java_config.target_dir());
        let java_toml: toml::Value = "[java]\nname = \"Test C\"\ntests_dir = \"path/to/test\"\nmain_class = \"OtherClass\"\ntimeout = false\ntarget_dir = \"testc/sub\"\n".parse().unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test C", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["OtherClass"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&None, java_config.case_timeout());
        assert_eq!("testc/sub", java_config.target_dir());
        let python_toml: toml::Value = "[python]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nversion = \"python3\"\nfile = \"source.py\"\ntarget_dir = \"testa/pysub\"\n".parse().unwrap();
        let python_config = TestConfig::from_toml_values(python_toml).unwrap();
        assert_eq!("Test A", python_config.name());
        assert_eq!(
            TestType::Directory("path/to/test"),
            python_config.test_type()
        );
        assert_eq!("python3", python_config.command());
        assert_eq!(&["source.py"], python_config.args());
        assert_eq!(&[""; 0], python_config.setup());
        assert_eq!(&Some(Duration::new(5, 0)), python_config.case_timeout());
        assert_eq!("testa/pysub", python_config.target_dir());
    }

    #[test]
    fn test_from_toml_with_args() {
        let java_config = TestConfig::from_toml_values(
            "[java]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nmain_class = \"Main\"\ntarget_dir = \"d\"\n"
                .parse()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(&["Main"], java_config.args());
        let java_config = TestConfig::from_toml_values(
            "[java]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nmain_class = \"Main\"\nargs = [\"Hello,\", \"world!\"]\ntarget_dir = \"d\"\n"
                .parse()
                .unwrap()
        ).unwrap();
        assert_eq!(&["Main", "Hello,", "world!"], java_config.args());
        let python_config = TestConfig::from_toml_values(
            "[python]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nfile = \"source.py\"\ntarget_dir = \"d\"\n"
                .parse()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(&["source.py"], python_config.args());
        let python_config = TestConfig::from_toml_values(
            "[python]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nfile = \"source.py\"\nargs = [\"Hello,\", \"world!\"]\ntarget_dir = \"d\"\n"
                .parse()
                .unwrap()
        ).unwrap();
        assert_eq!(&["source.py", "Hello,", "world!"], python_config.args());
    }
}
