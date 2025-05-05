use super::Expr;
use crate::context::Context;
use crate::itertools::ProductIterator;

fn outer(head: Expr, args: Vec<Expr>) -> Result<Expr, String> {
    let first_head = match args.get(0) {
        Some(Expr::Head(head, _)) => head.clone(),
        None => return Ok(Expr::Head(Box::new(head), vec![])),
        _ => return Err("Expected head expressions".to_string()),
    };

    // We don't care about heads as long as they're all the same. Extract arg vecs.
    // {{a, b, c}, {1, 2}, {x, y, z}} -> {a, 1, x}, {a, 1, y}, ...
    let arg_vecs = args
        .into_iter()
        .map(|a| match a {
            Expr::Head(head, vec) if head == first_head => Ok(vec),
            Expr::Head(h, _) => Err(format!(
                "Outer unexpected head. Got {}, expected {}",
                h, first_head
            )),
            o => Err(format!("Outer expected Head with args. Got {:?}", o)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    // First collect all tuples independently of structure
    let mut flat_hypercube: Vec<Expr> =
        ProductIterator::new(arg_vecs.iter().map(|a| a.len()).collect())
            .map(|idxs| {
                Expr::Head(
                    Box::new(head.clone()),
                    idxs.into_iter()
                        .enumerate()
                        .map(|(list_idx, item_idx)| arg_vecs[list_idx][item_idx].clone())
                        .collect(),
                )
            })
            .collect();

    // Use knowledge of list structures to fold dimensions
    for dim in (0..arg_vecs.len()).rev() {
        let dim_size = arg_vecs[dim].len();
        let mut folded_dim = Vec::new();
        let mut iter = flat_hypercube.into_iter();
        loop {
            // Exhaust the flat (on this dimension) grouping by dim-size
            let chunks: Vec<Expr> = iter.by_ref().take(dim_size).collect();
            if chunks.is_empty() {
                break;
            }
            folded_dim.push(Expr::from_head("List", chunks));
        }
        flat_hypercube = folded_dim;
    }

    assert!(flat_hypercube.len() == 1);
    Ok(flat_hypercube.swap_remove(0))
}

pub(crate) fn eval_outer(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    if args.len() < 1 {
        return Err("Outer must have at least 1 argument".to_string());
    }
    let head = args.remove(0);
    // Outer will produce new expressions that need to be evaluated
    super::evaluate(outer(head, args)?, ctx)
}

// fn distribute_op(lhs: Expr, rhs: Expr, op: &str, over: &str) -> Expr {
//     match lhs {
//         Expr::Head(h, lhs) if *h == Expr::Symbol(over.into()) => match rhs {
//             Expr::Head(h, rhs) if *h == Expr::Symbol(over.into()) => Expr::from_head(
//                 over,
//                 lhs.iter()
//                     .flat_map(|lhsi| {
//                         rhs.iter()
//                             .map(|rhsi| distribute_op(lhsi.clone(), rhsi.clone(), op, over))
//                     })
//                     .collect(),
//             ),
//             rhs => Expr::from_head(
//                 over,
//                 lhs.into_iter()
//                     .map(|lhsi| distribute_op(lhsi, rhs.clone(), op, over))
//                     .collect(),
//             ),
//         },
//         lhs => match rhs {
//             Expr::Head(h, rhs) if *h == Expr::Symbol(over.into()) => Expr::from_head(
//                 over,
//                 rhs.into_iter()
//                     .map(|rhsi| distribute_op(lhs.clone(), rhsi, op, over))
//                     .collect(),
//             ),
//             rhs => Expr::from_head(op, vec![lhs, rhs]),
//         },
//     }
// }

// fn flatten(expr: Expr, op: &str) -> Expr {
//     match expr {
//         Expr::Head(head, args) if *head == Expr::Symbol(op.into()) => Expr::Head(
//             head,
//             args.into_iter()
//                 .flat_map(|ai| match ai {
//                     Expr::Head(h, a) if *h == Expr::Symbol(op.into()) => {
//                         a.into_iter().map(|aj| flatten(aj, op)).collect()
//                     }
//                     other => vec![flatten(other, op)],
//                 })
//                 .collect(),
//         ),
//         Expr::Head(head, args) => {
//             Expr::Head(head, args.into_iter().map(|ai| flatten(ai, op)).collect())
//         }
//         other => other,
//     }
// }

// pub(crate) fn eval_times(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
//     // TODO: remove mut args do try_into

//     // distribute over List
//     // 3 * {a ,b} => {3 * a, 3 * b}
//     // {a, b} * 3 => {3 * a, 3 * b}
//     // {a, b} * {x, y} => {a * x, a * y, b * x, b * y}
//     // 3 * x => 3 * x
//     // 3 * x * {a, b} => {3 * x * a, 3 * x * b}
//     // 3 * {4, 5} => {12, 15}

//     // Distribute Times over List
//     let mut new_args = vec![args.remove(0)];
//     for rhs in args {
//         new_args = new_args
//             .into_iter()
//             .map(|lhsi| flatten(distribute_op(lhsi, rhs.clone(), "Times", "List"), "Times"))
//             .collect();
//     }
//     // Peel off wrapping Times result of distributing Times over List to avoid inf recurse
//     new_args = new_args
//         .into_iter()
//         .flat_map(|expr| match expr {
//             Expr::Head(h, a) if *h == Expr::Symbol("Times".into()) => a,
//             o => vec![o],
//         })
//         .collect();
//     // Run mmultiplication
//     let mut numeric: Option<f64> = None;
//     let mut new_args2 = Vec::new();
//     for arg in new_args {
//         // TODO: figure out how to properly think about this evaluate
//         match evaluate(arg, ctx)? {
//             Expr::Number(n) => *numeric.get_or_insert(1.0) *= n,
//             o => new_args2.push(o),
//         }
//     }
//     if numeric.is_some_and(|n| n != 1.0) || new_args2.len() == 0 {
//         new_args2.insert(0, Expr::Number(numeric.unwrap()));
//     }
//     if new_args2.len() == 1 {
//         Ok(new_args2.swap_remove(0))
//     } else {
//         Ok(Expr::from_head("Times", new_args2))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::expr::Expr;

//     #[test]
//     fn simple_multiplication() {
//         // test single arg
//         let args = vec![Expr::Number(3.0)];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(result, Expr::Number(3.0));

//         // test numerics
//         let args = vec![Expr::Number(3.0), Expr::Number(4.0)];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(result, Expr::Number(12.0));

//         // test identity
//         let args = vec![Expr::Number(1.0), Expr::Symbol("x".into())];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(result, Expr::Symbol("x".into()));

//         // test rhs multiplication
//         let args = vec![Expr::Symbol("x".into()), Expr::Number(1.0)];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(result, Expr::Symbol("x".into()));

//         // test symbol multiplication
//         let args = vec![Expr::Symbol("x".into()), Expr::Symbol("y".into())];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "Times",
//                 vec![Expr::Symbol("x".into()), Expr::Symbol("y".into())]
//             )
//         );
//     }

//     #[test]
//     fn distribute_over_list() {
//         use crate::expr::Expr::*;
//         // distribution of number on LHS
//         let args = vec![
//             Number(2.0),
//             Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head("List", vec![Number(6.0), Number(8.0)])
//         );

//         // distribution of sym on LHS
//         let args = vec![
//             Symbol("x".into()),
//             Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head("Times", vec![Number(3.0), Symbol("x".into())]),
//                     Expr::from_head("Times", vec![Number(4.0), Symbol("x".into())])
//                 ]
//             )
//         );

//         // distribution of sym on RHS
//         let args = vec![
//             Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
//             Symbol("x".into()),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head("Times", vec![Number(3.0), Symbol("x".into())]),
//                     Expr::from_head("Times", vec![Number(4.0), Symbol("x".into())])
//                 ]
//             )
//         );
//     }

//     #[test]
//     fn list_times_list() {
//         use crate::expr::Expr::*;

//         // {1, 2} * {x, y}
//         let args = vec![
//             Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Symbol("x".into()),
//                     Symbol("y".into()),
//                     Expr::from_head("Times", vec![Number(2.0), Symbol("x".into())]),
//                     Expr::from_head("Times", vec![Number(2.0), Symbol("y".into())]),
//                 ],
//             )
//         );

//         // {x, y} * {1, 2}
//         let args = vec![
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//             Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Symbol("x".into()),
//                     Expr::from_head("Times", vec![Number(2.0), Symbol("x".into())]),
//                     Symbol("y".into()),
//                     Expr::from_head("Times", vec![Number(2.0), Symbol("y".into())]),
//                 ],
//             )
//         );

//         // Check numeric lists
//         let args = vec![
//             Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
//             Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![Number(3.0), Number(4.0), Number(6.0), Number(8.0)]
//             )
//         );

//         // {x, y} * {w, z}
//         let args = vec![
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//             Expr::from_head("List", vec![Symbol("w".into()), Symbol("z".into())]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head("Times", vec![Symbol("x".into()), Symbol("w".into())]),
//                     Expr::from_head("Times", vec![Symbol("x".into()), Symbol("z".into())]),
//                     Expr::from_head("Times", vec![Symbol("y".into()), Symbol("w".into())]),
//                     Expr::from_head("Times", vec![Symbol("y".into()), Symbol("z".into())]),
//                 ],
//             )
//         );
//     }

//     #[test]
//     fn complex_distribution() {
//         use crate::expr::Expr::*;

//         // {x, y} * {w, z} * 3
//         let args = vec![
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//             Expr::from_head("List", vec![Symbol("w".into()), Symbol("z".into())]),
//             Number(3.0),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(3.0), Symbol("x".into()), Symbol("w".into())]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(3.0), Symbol("x".into()), Symbol("z".into())]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(3.0), Symbol("y".into()), Symbol("w".into())]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(3.0), Symbol("y".into()), Symbol("z".into())]
//                     ),
//                 ],
//             )
//         );

//         // 4 * {x, y} * {6, z} * 3
//         let args = vec![
//             Number(4.0),
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//             Expr::from_head("List", vec![Number(6.0), Symbol("z".into())]),
//             Number(3.0),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head("Times", vec![Number(72.0), Symbol("x".into())]),
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(12.0), Symbol("x".into()), Symbol("z".into())]
//                     ),
//                     Expr::from_head("Times", vec![Number(72.0), Symbol("y".into()),]),
//                     Expr::from_head(
//                         "Times",
//                         vec![Number(12.0), Symbol("y".into()), Symbol("z".into())]
//                     ),
//                 ],
//             )
//         );

//         // a * {x, y} * {w, z} * 3
//         let args = vec![
//             Symbol("a".into()),
//             Expr::from_head("List", vec![Symbol("x".into()), Symbol("y".into())]),
//             Expr::from_head("List", vec![Symbol("w".into()), Symbol("z".into())]),
//             Number(3.0),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head(
//                         "Times",
//                         vec![
//                             Number(3.0),
//                             Symbol("a".into()),
//                             Symbol("x".into()),
//                             Symbol("w".into())
//                         ]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![
//                             Number(3.0),
//                             Symbol("a".into()),
//                             Symbol("x".into()),
//                             Symbol("z".into())
//                         ]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![
//                             Number(3.0),
//                             Symbol("a".into()),
//                             Symbol("y".into()),
//                             Symbol("w".into())
//                         ]
//                     ),
//                     Expr::from_head(
//                         "Times",
//                         vec![
//                             Number(3.0),
//                             Symbol("a".into()),
//                             Symbol("y".into()),
//                             Symbol("z".into())
//                         ]
//                     ),
//                 ],
//             )
//         );
//     }

//     #[test]
//     fn nested_distribution() {
//         use crate::expr::Expr::*;

//         // {x, {a, b}} * {w, z}
//         let args = vec![
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Symbol("x".into()),
//                     Expr::from_head("List", vec![Symbol("a".into()), Symbol("b".into())]),
//                 ],
//             ),
//             Expr::from_head("List", vec![Symbol("w".into()), Symbol("z".into())]),
//         ];
//         let result = eval_times(args, &mut Context::new()).unwrap();
//         assert_eq!(
//             result,
//             Expr::from_head(
//                 "List",
//                 vec![
//                     Expr::from_head("Times", vec![Symbol("x".into()), Symbol("w".into())]),
//                     Expr::from_head("Times", vec![Symbol("x".into()), Symbol("z".into())]),
//                     Expr::from_head(
//                         "List",
//                         vec![
//                             Expr::from_head("Times", vec![Symbol("a".into()), Symbol("w".into())]),
//                             Expr::from_head("Times", vec![Symbol("b".into()), Symbol("w".into())]),
//                         ],
//                     ),
//                     Expr::from_head(
//                         "List",
//                         vec![
//                             Expr::from_head("Times", vec![Symbol("a".into()), Symbol("z".into())]),
//                             Expr::from_head("Times", vec![Symbol("b".into()), Symbol("z".into())]),
//                         ],
//                     )
//                 ],
//             )
//         );
//     }
// }
