use itertools::Itertools;
use prettytable::{Cell, Row};

use super::super::{ClassResults, TestAnswer};
use super::OutputMode;
/// An OutputMode which prints a table to some output stream
pub struct Table<T> {
    writer: T,
}

impl<T> Table<T> {
    pub fn with_output(writer: T) -> Self {
        Table { writer }
    }
}
impl Table<std::io::Stdout> {
    pub fn with_stdout() -> Self {
        Table::with_output(std::io::stdout())
    }
}

impl<T: std::io::Write> OutputMode for Table<T> {
    fn output_class_results(
        &mut self,
        results: &ClassResults,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let case_names: Vec<&String> = results
            .iter()
            .next()
            .expect("There weren't any test cases")
            .1
            .keys()
            .sorted()
            .collect();
        let mut table = prettytable::Table::new();
        let mut case_row = Row::empty();
        case_row.add_cell(Cell::new(""));
        case_row.add_cell(Cell::new("Passed"));
        case_row.add_cell(Cell::new("Total"));
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
            row.insert_cell(1, Cell::new(format!("{}", student_result.values().filter(|a| if let Ok(TestAnswer::Success) = a { true } else { false }).count()).as_str()));
            row.insert_cell(2, Cell::new(format!("{}", case_names.len()).as_str()));
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
        let mut writer = Table::with_output(Vec::<u8>::new());
        writer.output_class_results(&data).unwrap();
        let output = std::str::from_utf8(&writer.writer).unwrap();
        assert_eq!(output, "+-----------+--------+-------+--------+--------+--------+\n|           | Passed | Total | Case 1 | Case 2 | Case 3 |\n+-----------+--------+-------+--------+--------+--------+\n| Student A | 3      | 3     |        |        |        |\n+-----------+--------+-------+--------+--------+--------+\n| Student B | 1      | 3     |        | F      | T      |\n+-----------+--------+-------+--------+--------+--------+\n| Student C | 0      | 3     | C      | C      | C      |\n+-----------+--------+-------+--------+--------+--------+\n");
    }
}
