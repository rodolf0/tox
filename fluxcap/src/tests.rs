// Anchored spec
const ANCHORED_SPEC_TESTS: &[&str] = &[
    "monday",
    "monday january 1st",
    "monday 1st of january",
    "monday 1st of january 2020",
    "monday 3pm",
    "january",
    "january 2020",
    "january 1st",
    "january 1st 2020",
    "1st of january",
    "1st of january 2020",
    "2020",
    "3pm",
    "now",
    "today",
    "yesterday",
    "tomorrow",
];

const RELATIVE_SPEC_BASIC_TESTS: &[&str] = &[
    "this week",
    "next month",
    "last year",
    "a day ago",
    "3 weeks ago",
    "in a month",
    "in 2 years",
    // recurring tokens wihh non-constant lexemes
    "in 4 januaries",
    "3 feb 4ths ago",
    "last monday",
    "in 4 monday feb 27ths",
];

const RELATIVE_SPEC_ANCHORED_TESTS: &[&str] = &[
    "3 weeks before last",
    "2 months before next month",
    "2 months before a year ago",
    // relative_spec -> relative_anchor -> 'in' small_int recurring_token
    "4 days before in 2 weeks", // TODO: should we reject this ? nah it's valid

    // relative_spec -> ... relative_anchor -> relative_spec
    "2 months before 3 mondays hence",
    // relative_spec -> ... relative_anchor -> anchored_spec
    "2 months before january 1st 2020",
    // relative_spec -> ... relative_anchor -> scoped_spec
    "2 months before the 1st monday of january",
];

const SCOPED_SPEC_TESTS: &[&str] = &[
    // scoped_spec -> ... recurring_token
    "the 2nd week of january",
    "the last 2nd day of january",
    // scoped_spec -> ... anchored_spec
    "the 1st weekend of january 2020", // ambiguous with above
    // scoped_spec -> ... relative_spec
    "the 3rd monday of this year",
    // scoped_spec -> ... scoped_spec
    "the 2nd weekend of the 3rd month of next year",

];

const MULTI_SPEC_TESTS: &[&str] = &[
    // relative_spec -> scoped_spec -> anchored_spec
    "3 days after the 2nd week of january 2020",
    // relative_spec -> scoped_spec -> relative_spec
    "3 days after the 2nd week of next year",
    // relative_spec -> relative_spec -> scoped_spec
    "3 days after 2 weeks before the 2nd week of june",
    // relative_spec -> relative_spec -> anchored_spec
    "3 days after 2 weeks before january 1st 2020",
    // relative_spec -> relative_spec -> relative_spec
    "3 days after 2 weeks before a month ago",
    // scoped_spec -> relative_spec -> anchored_spec
    "the 2nd week of january after 3 days from now",
    // scoped_spec -> relative_spec -> scoped_spec
    "the 2nd week of january after 3 days from the 1st week of next month",
];

mod test {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
