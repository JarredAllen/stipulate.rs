use std::time::Duration;

/// Default timeout for java programs, in seconds, per test case
const DEFAULT_TIMEOUT: u64 = 5;

/// We need to compile their java files
const JAVA_SETUP: [&str; 1] = ["javac *.java"];

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
        let args = match conf.get("args") {
            None => Ok(vec![main_class]),
            Some(toml::Value::Array(_arr)) => Err("Custom arguments to main not yet supported"),
            _ => Err("\"args\", if specified, must be an array"),
        }?;
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
