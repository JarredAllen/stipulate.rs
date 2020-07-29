use itertools::Itertools;

use std::io::{self, Stdout, Write};

use super::super::{ClassResults, TestAnswer};
use super::OutputMode;

pub struct CsvOutput<T> {
    writer: T,
}
impl CsvOutput<Stdout> {
    pub fn with_stdout() -> Self {
        Self::with_output(io::stdout())
    }
}
impl<T> CsvOutput<T> {
    pub fn with_output(writer: T) -> Self {
        CsvOutput { writer }
    }
}

impl<T> OutputMode for CsvOutput<T>
where
    T: Write
{
    fn output_class_results(
            &mut self,
            results: &ClassResults,
        ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let case_names: Vec<String> = results
            .iter()
            .next()
            .expect("There weren't any test cases")
            .1
            .keys()
            .sorted()
            .cloned()
            .collect();
        write!(self.writer, "Name,Passed,Total,")?;
        writeln!(self.writer, "{}", case_names.join(","))?;
        for (student_name, student_result) in  results.iter().sorted_by_key(|a| a.0) {
            write!(self.writer, "{},{},{},", student_name, student_result.values().filter(|a| if let Ok(TestAnswer::Success) = a { true } else { false }).count(), case_names.len())?;
            let cases: Vec<_> = case_names.iter().map(|case| {
                match student_result.get(case).expect("Student missing test case in result") {
                    Ok(TestAnswer::Success) => " ",
                    Ok(TestAnswer::Failure) => "F",
                    Ok(TestAnswer::FailWithMessage(_)) => "F",
                    Ok(TestAnswer::CompileError) => "C",
                    Ok(TestAnswer::Timeout) => "T",
                    Err(_) => "!",
                }.to_string()
            }).collect();
            writeln!(self.writer, "{}", cases.join(","))?;
        }
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
        let mut writer = CsvOutput::with_output(Vec::<u8>::new());
        writer.output_class_results(&data).unwrap();
        let output = std::str::from_utf8(&writer.writer).unwrap();
        assert_eq!(output, "Name,Passed,Total,Case 1,Case 2,Case 3\nStudent A,3,3, , , \nStudent B,1,3, ,F,T\nStudent C,0,3,C,C,C\n");
    }
}
