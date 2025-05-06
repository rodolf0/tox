use super::Expr;
use crate::context::Context;
use crate::itertools::ProductIterator;

// Get the tensor shape of an expression
// Private somewhat fragile. Only looks at the first child
// Assumes Expr is a well formed tensor.
fn shape_of(expr: &Expr) -> Vec<usize> {
    let Expr::Head(_, args) = expr else {
        return vec![];
    };
    let mut shape = vec![args.len()];
    let mut args = args.first();
    while let Some(Expr::Head(_, children)) = args {
        shape.push(children.len());
        // ENHANCEMENT: check that all children have the same length
        args = children.first();
    }
    shape
}

fn reshape(head: &Expr, items: Vec<Expr>, shape: Vec<usize>) -> Result<Expr, String> {
    // Check we have the right amount of items for the shape
    if shape.iter().product::<usize>() != items.len() {
        return Err(format!(
            "Reshape failed: shape {:?} does not match items length {}",
            shape,
            items.len()
        ));
    }
    let mut stack = items;
    // Fold a flat list into the dimensions specified by the shape
    for d in (0..shape.len()).rev() {
        // Take these many elements, 'times' times to
        // create the items for this dimension.
        // Eg: Reshape flat list into {{x, y}, {1, 2, 3}, {a, b}}
        //     -> build 6 lists of 2 elements (take 2, 6 times)
        //     -> build 2 lists of 3 elements (take 3 of the above, 2 times)
        let take_count = shape[d];
        let times: usize = shape[..d].iter().product();
        let mut stack_iter = stack.into_iter();
        // Replace stack with nested items according to this dimension
        stack = (0..times)
            .map(|_| {
                Expr::Head(
                    Box::new(head.clone()), // Could use "List" but its more flexible
                    stack_iter.by_ref().take(take_count).collect(),
                )
            })
            .collect();
    }
    assert!(stack.len() == 1);
    Ok(stack.swap_remove(0))
}

pub(crate) fn eval_reshape(args: Vec<Expr>) -> Result<Expr, String> {
    let [items, shape]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("Reshape takes an Expr and a shape. {:?}", e))?;
    // Parse list of integers from shape Expression
    let shape = match shape {
        Expr::Head(h, args) if *h == Expr::Symbol("List".into()) => args
            .into_iter()
            .map(|s| match s {
                Expr::Number(i) => Ok(i as usize),
                o => return Err(format!("Shape must be a list of integers. Got {:?}", o)),
            })
            .collect::<Result<Vec<usize>, _>>()?,
        o => return Err(format!("Shape must be a list of integers. Got {:?}", o)),
    };
    // Could just use "List" but this is more powerful.
    // reshape(flatten(&Expr::Symbol("List".into()), vec![items]), shape)
    match eval_flatten(vec![items])? {
        Expr::Head(h, a) => reshape(&*h, a, shape),
        o => return Err(format!("Items must be a list. Got {:?}", o)),
    }
}

// Flattens a list of expressions only if they are of the same type as the head.
fn flatten(head: &Expr, items: Vec<Expr>) -> Vec<Expr> {
    items
        .into_iter()
        .flat_map(|item| match item {
            Expr::Head(h, args) if &*h == head => flatten(head, args),
            _ => vec![item],
        })
        .collect()
}

pub(crate) fn eval_flatten(args: Vec<Expr>) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Flatten expects a single arg. {:?}", e))?;
    match arg {
        Expr::Head(head, args) => Ok(Expr::Head(head.clone(), flatten(&*head, args))),
        _ => Err("Flatten expects a List/Head expression".to_string()),
    }
}

fn outer(head: Expr, args: Vec<Expr>) -> Result<Expr, String> {
    // Check 1st head to validate all args are the same head
    let first_head = match args.get(0) {
        Some(Expr::Head(head, _)) => head.clone(),
        None => return Ok(Expr::Head(Box::new(head), vec![])),
        _ => return Err("Expected list/head expressions".to_string()),
    };

    // Get the tensor shape of each arg of the Outer product
    let shape: Vec<usize> = args
        .iter()
        .flat_map(|a| match a {
            a @ Expr::Head(_, _) => shape_of(a),
            _ => vec![], // checked when collecting arg_vecs
        })
        .collect();

    // We don't care about heads as long as they're all the same. Extract flat items.
    // {{a, b, c}, {1, 2}, {x, y, z}} -> {a, 1, x}, {a, 1, y}, ...
    let flat_args = args
        .into_iter()
        .map(|a| match a {
            Expr::Head(head, vec) if head == first_head => Ok(flatten(&*first_head, vec)),
            Expr::Head(h, _) => Err(format!(
                "Outer unexpected head. Got {}, expected {}",
                h, first_head
            )),
            o => Err(format!("Outer expected Head with args. Got {:?}", o)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Calculate the flat outer-product values independent of structure
    let flat_counts = flat_args.iter().map(|av| av.len()).collect();
    let flat_hypercube: Vec<Expr> = ProductIterator::new(flat_counts)
        .map(|idxs| {
            Expr::Head(
                Box::new(head.clone()),
                idxs.into_iter()
                    .enumerate()
                    .map(|(list_idx, item_idx)| flat_args[list_idx][item_idx].clone())
                    .collect(),
            )
        })
        .collect();

    // Reshape the flat hypercube into the output tensor
    reshape(&*first_head, flat_hypercube, shape)
}

pub(crate) fn eval_outer(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    if args.len() < 1 {
        return Err("Outer must have at least 1 argument".to_string());
    }
    let head = args.remove(0);
    // Outer will produce new expressions that need to be evaluated
    super::evaluate(outer(head, args)?, ctx)
}

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
