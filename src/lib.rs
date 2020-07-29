pub mod conf;
pub mod output;
pub mod test;

pub use conf::TestConfig;
pub use test::{test_from_configuration, ClassResults, TestAnswer};
