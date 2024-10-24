#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Expr(String, Vec<Expr>),
    Symbol(String),
    Number(f64),
    Bool(bool),
    String(String),
}

impl Expr {
    pub fn head(&self) -> &str {
        match self {
            Expr::Expr(head, _) => head,
            Expr::Symbol(_) => "Symbol",
            Expr::Number(_) => "Number",
            Expr::Bool(_) => "Bool",
            Expr::String(_) => "String",
        }
    }
}

// Plus[a, Times[3, a, b]] a + 3 a b -> a (3 + b)
// Plus[b, Times[3, b]] b + 3 b -> b (4)
// Plus[b, Times[3, Plus[b, a]]] ... should expand, or factorize ?

pub fn evaluate(expr: Expr) -> Result<Expr, String> {
    match expr {
        Expr::Expr(head, mut args) => match head.as_ref() {
            "List" => {
                let mut evaled = Vec::new();
                for r in args.into_iter().map(|e| evaluate(e)) {
                    evaled.push(r?);
                }
                Ok(Expr::Expr(head, evaled))
            }
            "Rule" => {
                if args.len() != 2 {
                    Err("Rule must have 2 arguments".to_string())
                } else {
                    let lhs = args.remove(0);
                    let rhs = evaluate(args.remove(0))?;
                    Ok(Expr::Expr(head, vec![lhs, rhs]))
                }
            }
            "ReplaceAll" => {
                if args.len() != 2 {
                    Err("ReplaceAll must have 2 arguments".to_string())
                } else {
                    let rules = check_rules(args.remove(1))?;
                    replace_all(args.remove(0), &rules)
                }
            }
            other => panic!("{} head not implemented", other),
        },
        // Nothing specific on atomic expressions
        _ => Ok(expr),
    }
}

fn check_rules(rules: Expr) -> Result<Vec<(Expr, Expr)>, String> {
    // Evaluate rules and check they're Rule or List[Rule]
    match evaluate(rules)? {
        Expr::Expr(h, mut args) if h == "Rule" => {
            let lhs = args.remove(0);
            let rhs = args.remove(0);
            Ok(vec![(lhs, rhs)])
        }
        Expr::Expr(h, args) if h == "List" => {
            let mut rules = Vec::new();
            for arg in args {
                match arg {
                    Expr::Expr(head, mut args) if head == "Rule" => {
                        let lhs = args.remove(0);
                        let rhs = args.remove(0);
                        rules.push((lhs, rhs));
                    }
                    _ => {
                        return Err("ReplaceAll rules should be List[Rule]".to_string());
                    }
                }
            }
            Ok(rules)
        }
        other => Err(format!(
            "ReplaceAll rules should be Rule or List[Rule]. Found '{:?}'",
            other
        )),
    }
}

// ReplaceAll[x, Rule[x, 3]]
// ReplaceAll[List[1, 2, 3], Rule[List, FindRoot]]
fn replace_all(expr: Expr, rules: &[(Expr, Expr)]) -> Result<Expr, String> {
    match expr {
        // for each sub-expression apply the replacement
        Expr::Expr(head, args) => {
            let mut replaced = Vec::new();
            // First execute replace_all on subexpressions
            for r in args.into_iter().map(|a| replace_all(a, rules)) {
                replaced.push(r?);
            }
            // Check replacing the while re-written expression
            let mut replaced = Expr::Expr(head, replaced);
            for (lhs, rhs) in rules {
                if replaced == *lhs {
                    replaced = rhs.clone();
                }
            }
            // Match rules against head
            match replaced {
                Expr::Expr(head, args) => {
                    for r in rules {
                        if let (Expr::Symbol(lhs), Expr::Symbol(rhs)) = r {
                            if *lhs == head {
                                return Ok(Expr::Expr(rhs.clone(), args));
                            }
                        }
                    }
                    Ok(Expr::Expr(head, args))
                }
                other => Ok(other),
            }
        }
        atom => {
            for (lhs, rhs) in rules {
                if atom == *lhs {
                    return Ok(rhs.clone());
                }
            }
            Ok(atom) // no replacement
        }
    }
}

// FullForm can parse a string into an Expr
