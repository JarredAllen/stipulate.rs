use itertools::Itertools;
use prettytable::{Cell, Row, Table};

use super::super::{ClassResults, TestAnswer};
use super::OutputMode;
/// While there aren't any fields, don't rely on initializing it.
/// There may be fields added at any time
pub struct Print<T> {
    writer: T,
}

impl<T> Print<T> {
    pub fn with_stdout() -> Print<std::io::Stdout> {
        Print::with_output(std::io::stdout())
    }

    fn with_output(writer: T) -> Print<T> {
        Print { writer }
    }
}

impl<T: std::io::Write> OutputMode for Print<T> {
    fn output_class_results(
        &mut self,
        results: &ClassResults,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let case_names: Vec<&String> = results
            .iter()
            .next()
            .expect("Displaying an empty run")
            .1
            .keys()
            .sorted()
            .collect();
        let mut table = Table::new();
        let mut case_row = Row::empty();
        case_row.add_cell(Cell::new(""));
        for case in case_names.iter() {
            case_row.add_cell(Cell::new(case));
        }
        table.add_row(case_row);
        for (student_name, student_result) in results.iter().sorted_by_key(|a| a.0) {
            let mut row = Row::new(
                case_names
                    .iter()
                    .map(|case| {
                        Cell::new(
                            match student_result
                                .get(case.as_str())
                                .expect("Student missing case in their results")
                            {
                                Ok(TestAnswer::Success) => " ",
                                Ok(TestAnswer::Failure) => "F",
                                Ok(TestAnswer::FailWithMessage(_)) => "F",
                                Ok(TestAnswer::Timeout) => "T",
                                Ok(TestAnswer::CompileError) => "C",
                                Err(_) => "!",
                            },
                        )
                    })
                    .collect(),
            );
            row.insert_cell(0, Cell::new(student_name));
            table.add_row(row);
        }
        table.print(&mut self.writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn make_testing_data() -> ClassResults {
        let mut data = HashMap::new();
        let mut student_a = HashMap::new();
        student_a.insert(String::from("Case 1"), Ok(TestAnswer::Success));
        student_a.insert(String::from("Case 2"), Ok(TestAnswer::Success));
        student_a.insert(String::from("Case 3"), Ok(TestAnswer::Success));
        data.insert(String::from("Student A"), student_a);
        let mut student_b = HashMap::new();
        student_b.insert(String::from("Case 1"), Ok(TestAnswer::Success));
        student_b.insert(String::from("Case 2"), Ok(TestAnswer::Failure));
        student_b.insert(String::from("Case 3"), Ok(TestAnswer::Timeout));
        data.insert(String::from("Student B"), student_b);
        let mut student_c = HashMap::new();
        student_c.insert(String::from("Case 1"), Ok(TestAnswer::CompileError));
        student_c.insert(String::from("Case 2"), Ok(TestAnswer::CompileError));
        student_c.insert(String::from("Case 3"), Ok(TestAnswer::CompileError));
        data.insert(String::from("Student C"), student_c);
        data
    }

    #[test]
    fn test_print_output() {
        let data = make_testing_data();
        let mut writer = Print::with_output(Vec::<u8>::new());
        writer.output_class_results(&data).unwrap();
        let output = std::str::from_utf8(&writer.writer).unwrap();
        assert_eq!(output, "+-----------+--------+--------+--------+\n|           | Case 1 | Case 2 | Case 3 |\n+-----------+--------+--------+--------+\n| Student A |        |        |        |\n+-----------+--------+--------+--------+\n| Student B |        | F      | T      |\n+-----------+--------+--------+--------+\n| Student C | C      | C      | C      |\n+-----------+--------+--------+--------+\n");
    }
}
