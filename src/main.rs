use clap::{App, Arg};

use stipulate::output::{get_output_mode, get_output_mode_for_file};
use stipulate::{test_from_configuration, TestConfig};

fn main() {
    let args = App::new("stipulate.rs")
        .version("0.0.3")
        .author("Jarred Allen <jarredallen73@gmail.com>")
        .about("Automate testing of student code")
        .arg(
            Arg::with_name("config_file")
                .help("The file which stores the test configuration")
                .required(true),
        )
        .arg(
            Arg::with_name("output_method")
                .help("The method to use to output data")
                .required(true),
        )
        .arg(
            Arg::with_name("output_file")
                .help("The file to write output to (defaults to standard output")
                .short("o")
                .long("output-file")
                .takes_value(true)
        )
        .get_matches();
    let config_file = args.value_of("config_file").unwrap();
    let config = TestConfig::from_file(config_file).unwrap();
    let results = test_from_configuration(&config).unwrap();
    let output_method = args.value_of("output_method").unwrap();
    let mut output_writer = if let Some(output_file) = args.value_of("output_file") {
        get_output_mode_for_file(output_method, output_file).expect("Unknown output method")
    } else {
        get_output_mode(output_method).expect("Unknown output method")
    };
    output_writer.output_class_results(&results).unwrap();
}
