extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate toxearley as earley;
extern crate chrono;

use earley::Subtree;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;
use std::rc::Rc;

use chrono::*;

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


#[derive(PartialEq)]
pub enum Granularity {
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

#[derive(Clone, Debug)]
pub struct Range(DateTime<UTC>, Duration);

//struct Range<Tz: TimeZone> {
    //o: DateTime<UTC>, // origin
    //d: Duration,
    //g: Granularity,
//}

// need Rc cause I want to clone sequences
pub type Seq = Rc<Fn()->Box<Iterator<Item=Range>>>;

fn seq_dow(dow: usize) -> Seq {
    Rc::new(move || {
        let mut reftime = UTC::now().date();
        let dow_reftime = reftime.weekday().num_days_from_sunday() as usize;
        let diff = if dow < dow_reftime {
            (7 + dow - dow_reftime) % 7
        } else {
            dow - dow_reftime
        };
        for _ in 0..diff { reftime = reftime.succ(); }
        let reftime = reftime.and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range(reftime + Duration::days(x * 7), Duration::days(1))
        }))
    })
}

fn seq_month_of_year(moy: usize) -> Seq {
    Rc::new(move || {
        let mut a_month = UTC::now().date();
        Box::new((0..).map(move |_| {
            while (moy as u32) != a_month.month() { a_month = next_month(a_month); }
            let t0 = a_month.and_hms(0, 0, 0);
            a_month = next_month(a_month); // force advance
            let d0 = a_month.and_hms(0, 0, 0) - t0;
            Range(t0, d0)
        }))
    })
}

// 2 weeks after June 28th
//fn shift(origin: Seq, delta: Duration, g: Granularity) -> Seq {
//}

// ej: 28th day of month
// ej: 28th day of year
// ej: 28th week of year

// x = 2nd hour of the day
// 3rd x of the week
// the 3rd 2nd-hour-of-the-day of the week

// ej: 2nd 3-hour window within a fortnight
// ej: 2nd 28th-of-june a century
// needs sequences as arguments (instead of duraionts/granularities) because it returns sequences
// that can keep on yielding, example:
// 5th minute within an hour != 5th minute within 'this' hour
// the first is a sequence that we can ask
fn seq_nth(n: usize, win: Seq, within: Seq) -> Seq {
    // 1. take an instance of <within>
    // 2. cycle to the n-th instance if <win> within <within>
    // TODO: panic on win.duration > within.duration
    Rc::new(move || {
        const fuse: usize = 10000;
        // TODO: do we have to reset the <win> each time? maybe more efficient to carry on
        let win = win.clone();
        Box::new(within().take(fuse).filter_map(move |p| {
            let x = win().skip_while(|w| w.0 < p.0).nth(n - 1).unwrap();
            // TODO: restricting to sub-interval: change to takw_while?
            match (x.0 + x.1) <= (p.0 + p.1) {
                true => Some(x),
                false => None
            }
        }))
    })
}

fn intersect(a: Seq, b: Seq) -> Seq {
    Rc::new(move || {
        // make a the seq with shortest elem duration, b the one where these are largest
        //let mut a = a().peekable();
        //let mut b = b().peekable();
        //let (mut a, mut b) = match b.peek().unwrap().1 < a.peek().unwrap().1 {
            //true => (b, a),
            //false => (a, b)
        //};
        // TODO: verify not consuming 1st elem
        let x = a.clone()().next().unwrap();
        let y = b.clone()().next().unwrap();
        let (a, b) = match y.1 < x.1 {
            true => (b.clone(), a.clone()),
            false => (a.clone(), b.clone())
        };
        //unreachable!();
        //let a = a.clone();
        // TODO: not reseting <a> (and skipping to sync with next <b>) should we?
        Box::new(b().flat_map(move |x| {
            let x2 = x.clone();
            a().skip_while(move |y| y.0 < x.0)
             .take_while(move |y| (y.0 + y.1) <= (x2.0 + x2.1))
        }))
    })
}

fn seq_day() -> Seq {
    Rc::new(|| {
        let reftime = UTC::now().date().and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range(reftime + Duration::days(x), Duration::days(1))
        }))
    })
}

fn next_month<Tz: TimeZone>(mut d: Date<Tz>) -> Date<Tz> {
    let thismonth = d.month();
    while thismonth == d.month() { d = d.succ(); }
    d
}

fn next_year<Tz: TimeZone>(mut d: Date<Tz>) -> Date<Tz> {
    let thisyear = d.year();
    while thisyear == d.year() { d = d.succ(); }
    d
}

fn seq_month() -> Seq {
    Rc::new(|| { // TODO: this_month should be passed in probably
        let mut this_month = UTC::now().date().with_day(1).unwrap();
        Box::new((0..).map(move |_| {
            let t0 = this_month.and_hms(0, 0, 0);
            this_month = next_month(this_month);
            let d0 = this_month.and_hms(0, 0, 0) - t0;
            Range(t0, d0)
        }))
    })
}

fn seq_year() -> Seq {
    Rc::new(|| { // TODO: this_month should be passed in probably
        let mut this_year = UTC::now().date().with_day(1).unwrap().with_month(1).unwrap();
        Box::new((0..).map(move |x| {
            let t0 = this_year.and_hms(0, 0, 0);
            this_year = next_year(this_year);
            let d0 = this_year.and_hms(0, 0, 0) - t0;
            Range(t0, d0)
        }))
    })
}


//#[derive(Debug)]
pub enum Tobj {
    Duration(Duration),
    Seq(Seq),
    Range(Range),
    Num(i32),
}

//#[derive(Debug)]
pub struct TimeContext(Vec<Tobj>);


pub fn eval(ctx: &mut TimeContext, n: &Subtree) -> Tobj {
    match n {
        &Subtree::Node(ref sym, ref lexeme) => match sym.as_ref() {
            "<day-of-week>" => {
                let dow = day_of_week(lexeme).unwrap();
                Tobj::Seq(seq_dow(dow ))
            },
            "<ordinal>" => {
                let num = ordinals(lexeme).or(ordinal_digits(lexeme)).unwrap();
                Tobj::Num(num as i32)
            },
            "<named-month>" => {
                let month = month(lexeme).unwrap();
                Tobj::Seq(seq_month_of_year(month))
            },
            _ => panic!()
        },
        &Subtree::SubT(ref spec, ref subn) => match spec.as_ref() {
            "<time> -> <named-month> <ordinal>" => {
                let m = match eval(ctx, &subn[0]) {
                    Tobj::Seq(s) => s,
                    _ => panic!(),
                };
                let d = match eval(ctx, &subn[1]) {
                    Tobj::Num(n) => n,
                    _ => panic!(),
                };
                Tobj::Seq(intersect(m, seq_nth(d as usize, seq_day(), seq_month())))
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
    //let y = seq_nth(10, seq_day(), seq_month());
    //for x in seq_nth(5, y, seq_year())().take(5) {
    //for x in seq_day()().take(5) {
    //for x in seq_nth(5, seq_month(), seq_year())().take(5) {
    //let y = seq_dow(4);
    //for x in seq_nth(2, seq_dow(2), seq_month())().take(5) {
    //let y = seq_nth(3, seq_dow(4), seq_month());
    //let a = seq_month_of_year(8);
    //let b = seq_nth(3, seq_dow(4), seq_month());
    //let b = seq_nth(28, seq_day(), seq_month());
    //let y = intersect(a, b);
    //for x in y().take(5) {
        //println!("{} - {} - {}", x.0, x.1, (x.0 + x.1));
    //}
    //println!("=========");
    //for x in y().take(5) {
        //println!("{} - {} - {}", x.0, x.1, (x.0 + x.1));
    //}

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
                    match eval(&mut ctx, &tree) {
                        Tobj::Seq(s) => {
                            println!("{:?}", s().next().unwrap());
                        },
                        _ => panic!()
                    }
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
