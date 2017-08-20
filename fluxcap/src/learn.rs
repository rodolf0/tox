#![deny(warnings)]

use time_training::{TrainingSet, load_trainingset};

use earlgrey::Subtree;
use earlgrey;
use lexers;
use std::collections::HashMap;
use time_machine;

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
            w.get(spec).unwrap_or(&0.000_000_001).log2() +
                subn.iter().map(|st| score_tree(st, w)).sum::<f64>()
        },
        &Subtree::Leaf(ref sym, ref lexeme) => {
            let spec = format!("{} -> {}", sym, lexeme);
            w.get(&spec).unwrap_or(&0.000_000_001).log2()
        }
    }
}

pub fn learn(g: earlgrey::Grammar, td: &TrainingSet) -> HashMap<String, f64> {
    let p = earlgrey::EarleyParser::new(g);
    let mut name_count = HashMap::new();
    let mut rule_count = HashMap::new();

    // count occurences of rules eg: <S> -> <range>: 120
    for (ex, expected) in &td.examples {
        let mut tokens = lexers::DelimTokenizer::scanner(&ex, ", ", true);

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

pub fn learn(ts: &TrainingSet) -> HashMap<String, f64> {
    let mut rule_freq = HashMap::new();

    for (sample, value) in &ts.examples {
        for tree in ts.eval(ts.reftime, sample) {
        }
    }
}
