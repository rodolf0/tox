#[cfg(test)]
mod tests {
    use kronos::{Grain, TimeSpan};
    use serde::Deserialize;
    use time::{macros::format_description, PrimitiveDateTime, UtcOffset};
    use std::fs;

    #[derive(Deserialize)]
    struct Expected {
        start: String,
        end: String,
        grain: String,
    }

    #[derive(Deserialize)]
    struct TestCase {
        description: String,
        input: String,
        reftime: String,
        expected: Option<Expected>,
    }

    fn parse_time(s: &str) -> time::UtcDateTime {
        let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
        let dt = PrimitiveDateTime::parse(s, &format).unwrap();
        dt.assume_offset(UtcOffset::UTC).into()
    }

    fn parse_grain(s: &str) -> Grain {
        match s {
            "Second" => Grain::Second,
            "Minute" => Grain::Minute,
            "Hour" => Grain::Hour,
            "Day" => Grain::Day,
            "Week" => Grain::Day, // Weeks evaluate to Days internally in kronos
            "Month" => Grain::Month,
            "Year" => Grain::Year,
            _ => panic!("Unknown grain: {}", s),
        }
    }

    #[test]
    fn test_json_suite() {
        let json_data = fs::read_to_string("tests.json").unwrap();
        let tests: Vec<TestCase> = serde_json::from_str(&json_data).unwrap();

        let tm = crate::TimeMachine::new();

        for case in tests {
            let reftime = parse_time(&case.reftime);
            let mut results = tm.eval(&case.input, Some(reftime)).unwrap();
            
            match case.expected {
                Some(expected_val) => {
                    let result = results.remove(0);
                    let expected_start = parse_time(&expected_val.start);
                    let expected_end = parse_time(&expected_val.end);
                    let expected_grain = parse_grain(&expected_val.grain);

                    let expected = TimeSpan {
                        start: expected_start,
                        end: expected_end,
                        grain: expected_grain,
                    };

                    assert_eq!(
                        result, expected,
                        "Failed test: {}\nInput: '{}'\nExpected: {:?}\nGot: {:?}",
                        case.description, case.input, expected, result
                    );
                }
                None => {
                    assert!(
                        results.is_empty(),
                        "Failed test: {}\nInput: '{}'\nExpected empty results, got: {:?}",
                        case.description, case.input, results
                    );
                }
            }
        }
    }
}
