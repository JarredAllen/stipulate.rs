use std::time::Duration;

/// Default timeout for java programs, in seconds, per test case
const DEFAULT_TIMEOUT: u64 = 5;

/// Setup behavior: compile all java files at once
const JAVA_SETUP: [&str; 1] = ["javac *.java"];

/// This struct represents a configuration for running a java program.
///
/// See `JavaConfig::from_toml` for docs on how to create one.
pub struct JavaConfig {
    name: String,
    test_data_dir: String,
    timeout: Option<Duration>,
    // Stores the arguments. We never touch it directly, but args_refs
    // points within here, so we have to keep ownership
    #[allow(dead_code)]
    args: Vec<String>,
    // Stores a vec of pointers to str objects in args. Unsafe, but
    // needed to be able to produce a &[&str].
    args_refs: Vec<*const str>,
}

impl JavaConfig {
    /// Required fields in the toml:
    ///  - "name": A name for this test
    ///  - "tests_dir": The directory to contain input and output data
    ///  - "main_class": The class containing a public static void
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
            Some(toml::Value::Float(seconds)) => Ok(Some(Duration::new(
                *seconds as u64,
                ((seconds % 1.0) * 1e9) as u32,
            ))),
            None | Some(toml::Value::Boolean(true)) => Ok(Some(Duration::new(DEFAULT_TIMEOUT, 0))),
            Some(toml::Value::Boolean(false)) => Ok(None),
            _ => Err("\"timeout\", if specified, should be a number or false"),
        }?;
        let mut args: Vec<String> = match conf.get("args") {
            None => Ok(Vec::new()),
            Some(toml::Value::Array(arr)) => arr.iter().map(|v| match v {
                toml::Value::String(s) => Ok(s.clone()),
                toml::Value::Array(_) | toml::Value::Table(_) => Err("Args may not contain nested structures"),
                toml::Value::Integer(i) => Ok(format!("{}", i)),
                toml::Value::Float(f) => Ok(format!("{}", f)),
                toml::Value::Boolean(b) => Ok(format!("{}", b)),
                toml::Value::Datetime(d) => Ok(format!("{}", d)),
            }).collect(),
            _ => Err("\"args\", if specified, must be an array"),
        }?;
        args.insert(0, main_class);
        let args_refs: Vec<*const str> = args.iter().map(|s| s.as_str() as *const str).collect();
        Ok(JavaConfig {
            name,
            test_data_dir,
            timeout,
            args,
            args_refs,
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

    fn command(&self) -> &str {
        "java"
    }

    fn args(&self) -> &[&str] {
        // In this block, we pretend that args_refs was actually just
        // the Vec<&str> that the borrow checker doesn't let it be.
        unsafe { &*(self.args_refs.as_slice() as *const [*const str] as *const [&str]) }
    }

    fn setup(&self) -> &[&str] {
        &JAVA_SETUP
    }
}
