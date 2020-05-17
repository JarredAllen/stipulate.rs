use std::time::Duration;

/// Default timeout for python programs, in seconds, per test case
const DEFAULT_TIMEOUT: u64 = 5;

/// The default python interpreter to use, if unspecified
#[cfg(target_os = "windows")]
const DEFAULT_PYTHON: &str = "python";
#[cfg(any(target_os = "linux", target_os = "macos"))]
const DEFAULT_PYTHON: &str = "python3";

/// This struct represents a configuration for running a python program.
///
/// See `PythonConfig::from_toml` for docs on how to create one.
pub struct PythonConfig {
    name: String,
    test_data_dir: String,
    python_version: String,
    timeout: Option<Duration>,
    // Stores the arguments. We never touch it directly, but args_refs
    // points within here, so we have to keep ownership
    #[allow(dead_code)]
    args: Vec<String>,
    // Stores a vec of pointers to str objects in args. Unsafe, but
    // needed to be able to produce a &[&str].
    args_refs: Vec<*const str>,
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
    pub fn from_toml(conf: &toml::Value) -> Result<PythonConfig, &'static str> {
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
        let python_version = match conf.get("version") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Ok(String::from(DEFAULT_PYTHON)),
            _ => Err("\"version\", if specified, must be a string"),
        }?;
        let timeout = match conf.get("timeout") {
            Some(toml::Value::Integer(seconds)) => Ok(Some(Duration::new(*seconds as u64, 0))),
            Some(toml::Value::Float(seconds)) => Ok(Some(Duration::new(
                *seconds as u64,
                ((seconds % 1.0) * 1e9) as u32,
            ))),
            None | Some(toml::Value::Boolean(true)) => Ok(Some(Duration::new(DEFAULT_TIMEOUT, 0))),
            Some(toml::Value::Boolean(false)) => Ok(None),
            _ => Err("\"timeout\", if specified, should be a number or false"),
        }?;
        let main_file = match conf.get("file") {
            Some(toml::Value::String(s)) => Ok(s.clone()),
            None => Err("Missing \"file\" field"),
            _ => Err("\"file\" field should be a string"),
        }?;
        let mut args: Vec<String> = match conf.get("args") {
            None => Ok(Vec::new()),
            Some(toml::Value::Array(arr)) => arr
                .iter()
                .map(|v| match v {
                    toml::Value::String(s) => Ok(s.clone()),
                    toml::Value::Array(_) | toml::Value::Table(_) => {
                        Err("Args may not contain nested structures")
                    }
                    toml::Value::Integer(i) => Ok(format!("{}", i)),
                    toml::Value::Float(f) => Ok(format!("{}", f)),
                    toml::Value::Boolean(b) => Ok(format!("{}", b)),
                    toml::Value::Datetime(d) => Ok(format!("{}", d)),
                })
                .collect(),
            _ => Err("\"args\", if specified, must be an array"),
        }?;
        args.insert(0, main_file);
        let args_refs: Vec<*const str> = args.iter().map(|s| s.as_str() as *const str).collect();
        Ok(PythonConfig {
            name,
            test_data_dir,
            python_version,
            timeout,
            args,
            args_refs,
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

    fn command(&self) -> &str {
        &self.python_version
    }

    fn args(&self) -> &[&str] {
        // In this block, we pretend that args_refs was actually just
        // the Vec<&str> that the borrow checker doesn't let it be.
        unsafe { &*(self.args_refs.as_slice() as *const [*const str] as *const [&str]) }
    }

    fn setup(&self) -> &[&str] {
        // No need to do any setup
        &[]
    }
}
