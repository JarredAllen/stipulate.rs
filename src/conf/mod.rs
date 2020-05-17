mod java;
mod python;

use std::io::{self, Read};
use std::fs::File;
use std::time::Duration;

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
        let mut file = File::open(filename).map_err(|e| Error::IoError(e))?;
        let file_contents: toml::Value = read_from_stream(&mut file)?.parse().map_err(|e| Error::TomlError(e))?;
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
                            "java" => Box::new(java::JavaConfig::from_toml(value).map_err(|e| Error::OtherError(e.to_string()))?),
                            "python" => Box::new(python::PythonConfig::from_toml(value).map_err(|e| Error::OtherError(e.to_string()))?),
                            key => return Err(Error::OtherError(format!("Unrecognized config type: {}", key))),
                        }
                    })
                } else {
                    Err(Error::OtherError(String::from("The config file should have exactly one section")))
                }
            }
            _ => Err(Error::OtherError(String::from("The config file wasn't a table (shouldn't be thrown)"))),
        }
    }
}

pub trait Config {
    /// A name for this set of tests
    fn name(&self) -> &str;

    /// The folder containing the input and output files
    fn test_data_dir(&self) -> &str;

    /// The amount of time to let code run before timing out
    fn case_timeout(&self) -> &Option<Duration>;

    /// The name of the command to run.
    fn command(&self) -> &str;

    /// The arguments to be passed to the command.
    fn args(&self) -> &[String];

    /// A list of commands to be run in the student's code directory
    /// before running the code.
    fn setup(&self) -> &[String];
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
/// the results in a `String`.
fn read_from_stream<T: Read>(stream: &mut T) -> Result<String, Error> {
    let mut data = Vec::new();
    stream.read_to_end(&mut data).map_err(|e| Error::IoError(e))?;
    String::from_utf8(data).map_err(|e| Error::FromUtf8Error(e))
}
