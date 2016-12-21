use chrono::naive::datetime::NaiveDateTime as DateTime;
use earlgrey::Subtree;
use earlgrey;
use kronos;
use lexers;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use time;

#[derive(Debug)]
pub struct TrainData {
    reftime: DateTime,
    examples: HashMap<String, kronos::Range>,
}

pub fn load_training(path: &Path) -> Result<TrainData, Box<Error>> {
    use std::io::BufRead;
    // ignore lines starting with blanks
    // 1st line starting with 'ref:' indicates reference time
    // lines starting with " specify strings
    // other lines should be datetime fmt %Y-%m-%d %Y-%m-%d <grain>

    let mut f = try!(fs::File::open(path));
    let mut reader = io::BufReader::new(f);
    let mut data = HashMap::new();
    let mut sample_stack = Vec::new();
    let mut reftime = None;
    for line in reader.lines() {
        let line = try!(line);
        // skip empty lines
        if line.len() == 0 || line.starts_with(" ") {
            continue;
        }
        // reference time
        if line.starts_with("ref: ") {
            let rt = line.split(" ").skip(1).next().unwrap();
            let rt = format!("{} 00:00:00", rt);
            let rt = try!(DateTime::parse_from_str(&rt, "%Y-%m-%d %H:%M:%S"));
            reftime = Some(rt);
            continue;
        }
        // accumulate all samples
        if line.starts_with("\"") {
            let stripd = line[1..line.len()-1].to_string();
            sample_stack.push(stripd);
            continue;
        }
        // attribute this "range" to accumulated samples
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() != 3 {
            return Err(Box::new(
                io::Error::new(
                    io::ErrorKind::InvalidData, "Wrong number of args")));
        }
        let t0 = format!("{} 00:00:00", parts[0]);
        let t1 = format!("{} 00:00:00", parts[1]);
        let t0 = try!(DateTime::parse_from_str(&t0, "%Y-%m-%d %H:%M:%S"));
        let t1 = try!(DateTime::parse_from_str(&t1, "%Y-%m-%d %H:%M:%S"));
        let grain = kronos::Granularity::from(parts[2]);
        let r = kronos::Range{start: t0, end: t1, grain: grain};
        for sample in &sample_stack {
            data.insert(sample.clone(), r.clone());
        }
        sample_stack.clear();
    }
    Ok(TrainData{reftime: reftime.unwrap(), examples: data})
}

fn count_rules(tree: &Subtree,
               names: &mut HashMap<String, f64>,
               rules: &mut HashMap<String, f64>) {
    match tree {
        &Subtree::Node(ref spec, ref subn) => {
            // split rule name, TODO: Subtree should use Rule{}?
            let name = spec.splitn(2, " -> ").next().unwrap();
            *(names.entry(name.to_string()).or_insert(0.0)) += 1.0;
            *(rules.entry(spec.to_string()).or_insert(0.0)) += 1.0;
            for t in subn {
                count_rules(t, names, rules);
            }
        },
        &Subtree::Leaf(ref sym, ref lexeme) => {
            let spec = format!("{} -> {}", sym, lexeme);
            *(names.entry(sym.to_string()).or_insert(0.0)) += 1.0;
            *(rules.entry(spec).or_insert(0.0)) += 1.0;
        },
    }
}

pub fn score_tree(tree: &Subtree, w: &HashMap<String, f64>) -> f64 {
    match tree {
        &Subtree::Node(ref spec, ref subn) => {
            // split rule name, TODO: Subtree should use Rule{}?
            //let name = spec.splitn(2, " -> ").next().unwrap();
            //*(names.entry(name.to_string()).or_insert(0.0)) += 1.0;
            //*(rules.entry(spec.to_string()).or_insert(0.0)) += 1.0;
            //for t in subn {
                //count_rules(t, names, rules);
            //}
            w.get(spec).unwrap_or(&0.000_000_001).log2() +
                subn.iter().map(|st| score_tree(st, w)).sum::<f64>()
        },
        &Subtree::Leaf(ref sym, ref lexeme) => {
            let spec = format!("{} -> {}", sym, lexeme);
            w.get(&spec).unwrap_or(&0.000_000_001).log2()
        }
    }
}

pub fn learn(g: earlgrey::Grammar, td: &TrainData) -> HashMap<String, f64> {
    let p = earlgrey::EarleyParser::new(g);
    let mut name_count = HashMap::new();
    let mut rule_count = HashMap::new();

    // count occurences of rules eg: <S> -> <range>: 120
    for (ex, expected) in &td.examples {
        let mut tokens = lexers::DelimTokenizer::from_str(&ex, ", ", true);

        match p.parse(&mut tokens) {
            Err(err) => panic!("Learning error: {:?}", err),
            Ok(state) => {
                for tree in earlgrey::all_trees(p.g.start(), &state) {
                    // DEBUG: tree.print();
                    // TODO: get rid of this, use different grammars
                    // possibly pass in evaler ... eg: TimeMachine (Evaler trait)
                    let (spec, subn) = match tree {
                        Subtree::Node(spec, subn) => (spec.to_string(), subn),
                        _ => unreachable!("what!!")
                    };
                    // TODO: get rid of this, use different grammars
                    assert_eq!(spec, "<S> -> <range>");

                    let r0 = time::eval_range(td.reftime, &subn[0]);
                    // only bump counts on trees that match the expected result
                    if r0 == *expected {
                        count_rules(&subn[0], &mut name_count, &mut rule_count);
                    }
                }
            }
        }
    }

    // assign probability to rules as fraction of times the whole rule
    // is seen over the total expansions of the left-hand-side
    // AKA: maximum likelyhood estimates
    for (rule, count) in rule_count.iter_mut() {
        // TODO: split Subtree spec should be a Rule{}
        // needed to break Item::str_rule used by earlgrey::Subtree
        let name = rule.splitn(2, " -> ").next().unwrap();
        // will be non-0 because we just accounted for it
        let name_total = name_count.get(name).unwrap();
        *count = *count / *name_total;
    }

    rule_count
}


#[cfg(test)]
mod tests {
    use super::{load_training, learn};
    use std::path::PathBuf;
    use time::build_grammar;
    #[test]
    fn test_loading() {
        let mut traindata = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        traindata.push("src/time.train");
        assert!(load_training(&traindata).is_ok());
    }
    #[test]
    fn test_learning() {
        let mut traindata = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        traindata.push("src/time.train");
        let traindata = load_training(&traindata).unwrap();
        let histo = learn(build_grammar(), &traindata);
        println!("{:?}", histo);
    }
}
