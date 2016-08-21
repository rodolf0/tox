extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate toxearley as earley;
extern crate time;

use earley::Subtree;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;

fn day_of_week(d: &str) -> Option<usize> {
    let days: HashMap<&'static str, usize> = HashMap::from_iter(vec![
        "monday", "tuesday", "wednesday", "thursday",
        "friday", "saturday", "sunday"
    ].into_iter().enumerate().map(|(i, s)| (s, i+1)));
    days.get(d).cloned()
}

fn month(m: &str) -> Option<usize> {
    let months: HashMap<&str, usize> = HashMap::from_iter(vec![
        "january", "february", "march", "april", "may", "june",
        "july", "august", "september", "october", "november", "december"
    ].into_iter().enumerate().map(|(i, s)| (s, i+1)));
    months.get(m).cloned()
}


fn ordinals(n: &str) -> Option<usize> {
    let ord: HashMap<&str, usize> = HashMap::from_iter(vec![
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
        "eigth", "ninth", "thenth", "eleventh", "twelveth", "thirteenth",
        "fourteenth", "fifteenth", "sixteenth", "seventeenth", "eighteenth",
        "nineteenth", "twentieth", "twenty-first", "twenty-second",
        "twenty-third", "twenty-fourth", "twenty-fith", "twenty-sixth",
        "twenty-seventh", "twenty-eigth", "twenty-ninth", "thirtieth",
        "thirty-first",
    ].into_iter().enumerate().map(|(i, s)| (s, i+1)));
    ord.get(n).cloned()
}

fn ordinal_digits(n: &str) -> Option<usize> {
    let ord = Regex::new(r"(\d+) ?(?:st|nd|rd|th)").unwrap();
    if let Some(caps) = ord.captures(n) {
        return caps.at(1).map(|num| usize::from_str(num).unwrap())
    }
    None
}

// https://github.com/wit-ai/duckling/blob/master/resources/languages/en/rules/time.clj
fn build_grammar() -> earley::Grammar {
    let gb = earley::GrammarBuilder::new()
      .symbol(("<day-of-week>", |d: &str| day_of_week(d).is_some()))
      .symbol(("<ordinal (digit)>", |d: &str| ordinal_digits(d).is_some()))
      .symbol(("<ordinal (names)>", |d: &str| ordinals(d).is_some()))
      .symbol(("<ordinal>", |n: &str| ordinals(n).is_some() || ordinal_digits(n).is_some()))
      .symbol(("<named-month>", |m: &str| month(m).is_some()))
      ;

    // misc symbols
    let gb = gb.symbol(("this", |n: &str| n == "this"))
      .symbol(("next", |n: &str| n == "next"))
      .symbol(("the", |n: &str| n == "the"))
      .symbol(("last", |n: &str| n == "last"))
      .symbol(("before", |n: &str| n == "before"))
      .symbol(("after", |n: &str| n == "after"))
      .symbol(("of", |n: &str| n == "of"))
      .symbol(("now", |n: &str| n == "now"))
      .symbol(("today", |n: &str| n == "today"))
      .symbol(("tomorrow", |n: &str| n == "tomorrow"))
      .symbol(("yesterday", |n: &str| n == "yesterday"))
      .symbol(("year", |n: &str| n == "year"))
      ;

    let gb = gb.symbol("<time>")
      ;

    let gb = gb.rule("<time>", &["<time>", "<time>"])        // intersect 2 times
      .rule("<time>", &["<named-month>"])                    // march
      .rule("<time>", &["year"])                             // march

      .rule("<time>", &["<day-of-week>"])                    // thursday
      .rule("<time>", &["this", "<day-of-week>"])            // next tuesday
      .rule("<time>", &["next", "<day-of-week>"])            // next tuesday

      .rule("<time>", &["last", "<time>"])                   // last week | last sunday | last friday
      .rule("<time>", &["next", "<time>"])                   // last week | last sunday | last friday
      .rule("<time>", &["the", "<ordinal>"])                 // the 2nd
      .rule("<time>", &["<named-month>", "<ordinal>"])
      .rule("<time>", &["<ordinal>", "<time>", "of", "<time>"])
      .rule("<time>", &["<time>", "before", "last"])
      .rule("<time>", &["<time>", "after", "next"])

      .rule("<time>", &["<ordinal>", "<time>", "after", "<time>"])
      .rule("<time>", &["<ordinal>", "<time>", "of", "<time>"])
      .rule("<time>", &["the", "<ordinal>", "<time>", "of", "<time>"])
      ;

    gb.into_grammar("<time>")
}


pub enum Duration {
    Second,
    Minute,
    Hour,
    TimeOfDay, // ??
    Day,
    Month,
    Season,
    Quarter,
    Weekend,
    Week,
    Year,
    Decade,
    Century,
    TempD, // constante dependent duration
}

pub struct Range(time::Tm, time::Tm);

// time functions
fn seq(g: Duration) -> Box<Iterator<Item=Range>> {
    // return a function that returns ranges? (ie: for a Duration::Day return a func that
    // returns day intervals
    panic!("not implemented")
}

fn trunctm(mut t: time::Tm, g: Duration) -> time::Tm {
    t.tm_sec = 0;
    t.tm_min = 0;
    t.tm_hour = 0;
    t.tm_nsec = 0;
    t
}

type Seq = Iterator<Item=(time::Tm, time::Tm)>;

fn seq_dow(dow: usize) -> Box<Seq> {
    let reftime = time::now();
    let diff = (dow as i32 - reftime.tm_wday) % 7;
    let reftime = trunctm(reftime + time::Duration::days(diff as i64), Duration::Day);
    Box::new((0..).map(move |x| {
        (reftime + time::Duration::days(x * 7), reftime + time::Duration::days(x * 7 + 1))
    }))
}

//fn deq_nth(n: usize, )

//pub Monday = Sequence{granularity: Duration::Day, gen: XX};

#[derive(Debug)]
pub enum Telem {
    Duration(String),
    Sequence(String), // set of ranges with identical granularity, eg: thursday (all possible thursdays)
    Range(time::Tm, time::Tm),
    Number(i32),
}

#[derive(Debug)]
pub struct TimeContext(Vec<Telem>);

pub fn nth() {}
pub fn intersect() {}
pub fn shift() {}
pub fn next() {}
pub fn prev() {}
pub fn nearest_fwd() {}
pub fn nearset_bck() {}


pub fn eval(ctx: &mut TimeContext, n: &Subtree) -> Option<Telem> {
    match n {
        &Subtree::Node(ref sym, ref lexeme) => match sym.as_ref() {
            "<day-of-week>" => {
                //let dow = day_of_week(lexeme).unwrap();
                //seq(Duration::Day, )
                Some(Telem::Sequence(lexeme.clone()))
            },
            "<ordinal>" => {
                let num = ordinals(lexeme).or(ordinal_digits(lexeme)).unwrap();
                Some(Telem::Number(num as i32))
            },
            "<named-month>" => {
                Some(Telem::Sequence(lexeme.clone()))
            },
            _ => panic!()
        },
        &Subtree::SubT(ref spec, ref subn) => match spec.as_ref() {
            "<time> -> this <day-of-week>" |
            "<time> -> next <day-of-week>" => {
                panic!()
            },
            "<time> -> <day-of-week>" => {
                panic!()
            },
            "<time> -> <named-month> <ordinal>" => {
                let m = eval(ctx, &subn[0]).unwrap();
                let d = eval(ctx, &subn[1]).unwrap();
                Some(m)
                //println!("what !! {:?} {:?}", m, d);
            },
            _ => panic!()
        }
    }
}


fn dotprinter(node: &Subtree, n: usize) {
    match node {
        &Subtree::Node(ref term, ref value) => println!("  \"{}. {}\" -> \"{}. {}\"", n, term, n + 1, value),
        &Subtree::SubT(ref spec, ref childs) => for (nn, c) in childs.iter().enumerate() {
            let x = match c {
                &Subtree::Node(ref term, _) => term,
                &Subtree::SubT(ref sspec, _) => sspec,
            };
            println!("  \"{}. {}\" -> \"{}. {}\"", n, spec, n + nn + 100, x);
            dotprinter(&c, n + nn + 100);
        }
    }
}

fn main() {
    for x in seq_dow(2).take(5) {
        println!("{} - {}", x.0.asctime(), x.1.asctime());
    }

    let parser = earley::EarleyParser::new(build_grammar());

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).
            collect::<Vec<String>>().join(" ");
        match parser.parse(&mut lexers::DelimTokenizer::from_str(&input, " ", true)) {
            Ok(estate) => {
                //let tree = earley::one_tree(parser.g.start(), &estate);
                for tree in earley::all_trees(parser.g.start(), &estate) {
                    println!("digraph x {{");
                    dotprinter(&tree, 0);
                    println!("}}");

                    let mut ctx = TimeContext(Vec::new());
                    println!("{:?}", eval(&mut ctx, &tree));
                }
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
        return;
    }

    //let mut ctx = HashMap::new();
    //while let Some(input) = linenoise::input("~> ") {
        //linenoise::history_add(&input[..]);
        //match parser.parse(&mut lexers::DelimTokenizer::from_str(&input, " ", true)) {
            //Ok(estate) => {
                //let tree = earley::one_tree(parser.g.start(), &estate);
                //let val = xeval(&tree, &mut ctx)[0];
                //println!("{:?}", val);
            //},
            //Err(e) => println!("Parse err: {:?}", e)
        //}
    //}
}
