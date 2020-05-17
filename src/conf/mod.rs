mod java;
mod python;

use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;

use std::ops::Deref;
use std::ops::DerefMut;

/// This struct represents all of the configuration for a test run. This
/// entire module is about creating, exporting, and dealing with these.
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
    pub fn from_file(filename: &str) -> Result<TestConfig, Error> {
        let mut file = File::open(filename).map_err(Error::IoError)?;
        let file_contents: toml::Value = read_from_stream(&mut file)?
            .parse()
            .map_err(Error::TomlError)?;
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
    /// Configuration options for java are at
    /// `java::JavaConfig::from_toml`.
    ///
    /// Configuration options for python are at
    /// `python::PythonConfig::from_toml`.
    pub fn from_toml_values(values: toml::Value) -> Result<TestConfig, Error> {
        match values {
            toml::Value::Table(table) => {
                if table.len() == 1 {
                    let key = table.keys().find(|_| true).unwrap();
                    let value = table.get(key).unwrap();
                    Ok(TestConfig {
                        config: match key.as_str() {
                            "java" => Box::new(
                                java::JavaConfig::from_toml(value)
                                    .map_err(|e| Error::OtherError(e.to_string()))?,
                            ),
                            "python" => Box::new(
                                python::PythonConfig::from_toml(value)
                                    .map_err(|e| Error::OtherError(e.to_string()))?,
                            ),
                            key => {
                                return Err(Error::OtherError(format!(
                                    "Unrecognized config type: {}",
                                    key
                                )))
                            }
                        },
                    })
                } else {
                    Err(Error::OtherError(String::from(
                        "The config file should have exactly one section",
                    )))
                }
            }
            _ => Err(Error::OtherError(String::from(
                "The config file wasn't a table (shouldn't be thrown)",
            ))),
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
}

/// The different kinds of tests that can be done.
///
/// Currently, it just supports having a directory with inputs and
/// outputs. I plan to eventually support other options (including
/// junit).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestType<'a> {
    Directory(&'a str),
}

/// An enum representing the kinds of errors that can be encountered
/// when trying to read a `Config` object from a file or write it to a
/// file.
#[derive(Debug)]
pub enum Error {
    TomlError(toml::de::Error),
    IoError(io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    OtherError(String),
}

/// Reads from an input stream until the input stream ends, and returns
/// the results in a `String`, decoded as UTF8.
fn read_from_stream<T: Read>(stream: &mut T) -> Result<String, Error> {
    let mut data = Vec::new();
    stream.read_to_end(&mut data).map_err(Error::IoError)?;
    String::from_utf8(data).map_err(Error::FromUtf8Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_toml() {
        let java_toml: toml::Value =
            "[java]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nmain_class = \"Main\"\n"
                .parse()
                .unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test A", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["Main"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&Some(Duration::new(5, 0)), java_config.case_timeout());
        let java_toml: toml::Value = "[java]\nname = \"Test B\"\ntests_dir = \"path/to/test\"\nmain_class = \"MainB\"\ntimeout = 1".parse().unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test B", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["MainB"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&Some(Duration::new(1, 0)), java_config.case_timeout());
        let java_toml: toml::Value = "[java]\nname = \"Test C\"\ntests_dir = \"path/to/test\"\nmain_class = \"OtherClass\"\ntimeout = false".parse().unwrap();
        let java_config = TestConfig::from_toml_values(java_toml).unwrap();
        assert_eq!("Test C", java_config.name());
        assert_eq!(TestType::Directory("path/to/test"), java_config.test_type());
        assert_eq!("java", java_config.command());
        assert_eq!(&["OtherClass"], java_config.args());
        assert_eq!(&["javac *.java"], java_config.setup());
        assert_eq!(&None, java_config.case_timeout());
        let python_toml: toml::Value = "[python]\nname = \"Test A\"\ntests_dir = \"path/to/test\"\nversion = \"python3\"\nfile = \"source.py\"\n".parse().unwrap();
        let python_config = TestConfig::from_toml_values(python_toml).unwrap();
        assert_eq!("Test A", python_config.name());
        assert_eq!(TestType::Directory("path/to/test"), python_config.test_type());
        assert_eq!("python3", python_config.command());
        assert_eq!(&["source.py"], python_config.args());
        assert_eq!(&[""; 0], python_config.setup());
        assert_eq!(&Some(Duration::new(5, 0)), python_config.case_timeout());
    }
}
