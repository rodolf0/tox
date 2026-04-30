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
    struct ExpectedCount {
        unit: String,
        span: Option<Expected>,
        total: f64,
        full_spans: usize,
    }

    #[derive(Deserialize)]
    struct TestCase {
        description: String,
        input: String,
        reftime: String,
        expected: Option<Expected>,
        expected_count: Option<ExpectedCount>,
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
            let eval_result = tm.eval(&case.input, Some(reftime));
            
            if case.expected.is_none() && case.expected_count.is_none() {
                if let Ok(results) = eval_result {
                    assert!(
                        results.is_empty(),
                        "Failed test: {}\nInput: '{}'\nExpected empty results, got: {:?}",
                        case.description, case.input, results
                    );
                }
                continue;
            }
            
            let mut results = eval_result.unwrap();
            
            if let Some(expected_val) = case.expected {
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
                    result, crate::time_semantics::TimeResult::Span(expected.clone()),
                    "Failed test: {}\nInput: '{}'\nExpected: {:?}\nGot: {:?}",
                    case.description, case.input, expected, result
                );
            } else if let Some(expected_count) = case.expected_count {
                let result = results.remove(0);
                
                if let crate::time_semantics::TimeResult::Count(actual_count) = &result {
                    assert_eq!(
                        actual_count.unit, expected_count.unit,
                        "Failed test: {}\nInput: '{}'\nExpected unit: {}\nGot: {}",
                        case.description, case.input, expected_count.unit, actual_count.unit
                    );
                    assert_eq!(
                        actual_count.full_spans, expected_count.full_spans,
                        "Failed test: {}\nInput: '{}'\nExpected full_spans: {}\nGot: {}",
                        case.description, case.input, expected_count.full_spans, actual_count.full_spans
                    );
                    // Use epsilon for float comparison
                    assert!(
                        (actual_count.total - expected_count.total).abs() < 1e-6,
                        "Failed test: {}\nInput: '{}'\nExpected total: {}\nGot: {}",
                        case.description, case.input, expected_count.total, actual_count.total
                    );
                    
                    if let Some(expected_span) = expected_count.span {
                        let expected_start = parse_time(&expected_span.start);
                        let expected_end = parse_time(&expected_span.end);
                        let expected_grain = parse_grain(&expected_span.grain);
                        let span = TimeSpan {
                            start: expected_start,
                            end: expected_end,
                            grain: expected_grain,
                        };
                        assert_eq!(
                            actual_count.span, span,
                            "Failed test: {}\nInput: '{}'\nExpected span: {:?}\nGot: {:?}",
                            case.description, case.input, span, actual_count.span
                        );
                    }
                } else {
                    panic!("Failed test: {}\nInput: '{}'\nExpected CountResult, got: {:?}", case.description, case.input, result);
                }
            } else {
                assert!(
                    results.is_empty(),
                    "Failed test: {}\nInput: '{}'\nExpected empty results, got: {:?}",
                    case.description, case.input, results
                );
            }
        }
    }
}
