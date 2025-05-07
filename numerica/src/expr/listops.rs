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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::expr::Expr::*;

    #[test]
    fn shapes() {
        // {}
        let t = Expr::from_head("List", vec![]);
        assert_eq!(shape_of(&t), vec![0]);

        // {1, 2, 3, 4}
        let t = Expr::from_head(
            "List",
            vec![Number(1.0), Number(2.0), Number(3.0), Number(4.0)],
        );
        assert_eq!(shape_of(&t), vec![4]);

        // {{1, 2}, {3, 4}, {5, 6}}
        let t = Expr::from_head(
            "List",
            vec![
                Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
                Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
                Expr::from_head("List", vec![Number(5.0), Number(6.0)]),
            ],
        );
        assert_eq!(shape_of(&t), vec![3, 2]);

        // {{{1, 2, 3}, {4, 5, 6}}, {{7, 8, 9}, {10, 11, 12}}}
        let t = Expr::from_head(
            "List",
            vec![
                Expr::from_head(
                    "List",
                    vec![
                        Expr::from_head("List", vec![Number(1.0), Number(2.0), Number(3.0)]),
                        Expr::from_head("List", vec![Number(4.0), Number(5.0), Number(6.0)]),
                    ],
                ),
                Expr::from_head(
                    "List",
                    vec![
                        Expr::from_head("List", vec![Number(7.0), Number(8.0), Number(9.0)]),
                        Expr::from_head("List", vec![Number(10.0), Number(11.0), Number(12.0)]),
                    ],
                ),
            ],
        );
        assert_eq!(shape_of(&t), vec![2, 2, 3]);
    }

    #[test]
    fn flatten_reshape() {
        let head = Expr::Symbol("List".into());
        // {}
        let flat = flatten(&head, vec![]);
        assert_eq!(&flat, &vec![]);
        assert_eq!(
            reshape(&head, flat, vec![0]),
            Ok(Expr::from_head("List", vec![]))
        );

        // {1, 2, 3, 4}
        let elems = vec![Number(1.0), Number(2.0), Number(3.0), Number(4.0)];
        let flat = flatten(&head, elems.clone());
        assert_eq!(&flat, &elems);
        assert_eq!(
            reshape(&head, flat, vec![4]),
            Ok(Expr::from_head("List", elems))
        );

        // {{1, 2}, {3, 4}, {5, 6}}
        let elems = vec![
            Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
            Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
            Expr::from_head("List", vec![Number(5.0), Number(6.0)]),
        ];
        let flat = flatten(&head, elems.clone());
        assert_eq!(
            &flat,
            &vec![
                Number(1.0),
                Number(2.0),
                Number(3.0),
                Number(4.0),
                Number(5.0),
                Number(6.0)
            ]
        );
        assert_eq!(
            reshape(&head, flat.clone(), vec![3, 2]),
            Ok(Expr::from_head("List", elems))
        );
        assert_eq!(
            reshape(&head, flat, vec![2, 3]),
            Ok(Expr::from_head(
                "List",
                vec![
                    Expr::from_head("List", vec![Number(1.0), Number(2.0), Number(3.0)]),
                    Expr::from_head("List", vec![Number(4.0), Number(5.0), Number(6.0)]),
                ]
            ))
        );

        // {{{1, 2, 3}, {4, 5, 6}}, {{7, 8, 9}, {10, 11, 12}}}
        let elems = vec![
            Expr::from_head(
                "List",
                vec![
                    Expr::from_head("List", vec![Number(1.0), Number(2.0), Number(3.0)]),
                    Expr::from_head("List", vec![Number(4.0), Number(5.0), Number(6.0)]),
                ],
            ),
            Expr::from_head(
                "List",
                vec![
                    Expr::from_head("List", vec![Number(7.0), Number(8.0), Number(9.0)]),
                    Expr::from_head("List", vec![Number(10.0), Number(11.0), Number(12.0)]),
                ],
            ),
        ];
        let flat = flatten(&head, elems.clone());
        assert_eq!(
            &flat,
            &vec![
                Number(1.0),
                Number(2.0),
                Number(3.0),
                Number(4.0),
                Number(5.0),
                Number(6.0),
                Number(7.0),
                Number(8.0),
                Number(9.0),
                Number(10.0),
                Number(11.0),
                Number(12.0),
            ]
        );
        assert_eq!(
            reshape(&head, flat, vec![2, 2, 3]),
            Ok(Expr::from_head("List", elems))
        );
    }

    #[test]
    fn outer_basic() {
        assert_eq!(
            outer(Expr::Symbol("Times".into()), vec![]),
            Ok(Expr::from_head("Times", vec![]))
        );

        // Outer[f, {a, b}, {x, y, z}]
        assert_eq!(
            outer(
                Expr::Symbol("Times".into()),
                vec![
                    Expr::from_head("List", vec![Symbol("a".into()), Symbol("b".into())]),
                    Expr::from_head(
                        "List",
                        vec![Symbol("x".into()), Symbol("y".into()), Symbol("z".into())]
                    ),
                ]
            ),
            Ok(Expr::from_head(
                "List",
                vec![
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Symbol("a".into()), Symbol("x".into())]),
                            Expr::from_head("Times", vec![Symbol("a".into()), Symbol("y".into())]),
                            Expr::from_head("Times", vec![Symbol("a".into()), Symbol("z".into())]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Symbol("b".into()), Symbol("x".into())]),
                            Expr::from_head("Times", vec![Symbol("b".into()), Symbol("y".into())]),
                            Expr::from_head("Times", vec![Symbol("b".into()), Symbol("z".into())]),
                        ]
                    )
                ]
            ))
        );
    }

    #[test]
    fn outer_vector() {
        // Outer[Times, {1, 2, 3, 4}, {a, b, c}]
        assert_eq!(
            outer(
                Expr::Symbol("Times".into()),
                vec![
                    Expr::from_head(
                        "List",
                        vec![Number(1.0), Number(2.0), Number(3.0), Number(4.0)]
                    ),
                    Expr::from_head(
                        "List",
                        vec![Symbol("a".into()), Symbol("b".into()), Symbol("c".into())]
                    ),
                ]
            ),
            Ok(Expr::from_head(
                "List",
                vec![
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Number(1.0), Symbol("a".into())]),
                            Expr::from_head("Times", vec![Number(1.0), Symbol("b".into())]),
                            Expr::from_head("Times", vec![Number(1.0), Symbol("c".into())]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Number(2.0), Symbol("a".into())]),
                            Expr::from_head("Times", vec![Number(2.0), Symbol("b".into())]),
                            Expr::from_head("Times", vec![Number(2.0), Symbol("c".into())]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Number(3.0), Symbol("a".into())]),
                            Expr::from_head("Times", vec![Number(3.0), Symbol("b".into())]),
                            Expr::from_head("Times", vec![Number(3.0), Symbol("c".into())]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("Times", vec![Number(4.0), Symbol("a".into())]),
                            Expr::from_head("Times", vec![Number(4.0), Symbol("b".into())]),
                            Expr::from_head("Times", vec![Number(4.0), Symbol("c".into())]),
                        ]
                    ),
                ]
            ))
        );
    }

    #[test]
    fn outer_matrix() {
        // Outer[Times, {{1, 2}, {3, 4}}, {{a, b}, {c, d}}]
        assert_eq!(
            outer(
                Expr::Symbol("Times".into()),
                vec![
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("List", vec![Number(1.0), Number(2.0)]),
                            Expr::from_head("List", vec![Number(3.0), Number(4.0)]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("List", vec![Symbol("a".into()), Symbol("b".into())]),
                            Expr::from_head("List", vec![Symbol("c".into()), Symbol("d".into())]),
                        ]
                    ),
                ]
            ),
            Ok(Expr::from_head(
                "List",
                vec![
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head(
                                "List",
                                vec![
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(1.0), Symbol("a".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(1.0), Symbol("b".into())]
                                            ),
                                        ]
                                    ),
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(1.0), Symbol("c".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(1.0), Symbol("d".into())]
                                            ),
                                        ]
                                    ),
                                ]
                            ),
                            Expr::from_head(
                                "List",
                                vec![
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(2.0), Symbol("a".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(2.0), Symbol("b".into())]
                                            ),
                                        ]
                                    ),
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(2.0), Symbol("c".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(2.0), Symbol("d".into())]
                                            ),
                                        ]
                                    ),
                                ]
                            )
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head(
                                "List",
                                vec![
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(3.0), Symbol("a".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(3.0), Symbol("b".into())]
                                            ),
                                        ]
                                    ),
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(3.0), Symbol("c".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(3.0), Symbol("d".into())]
                                            ),
                                        ]
                                    ),
                                ]
                            ),
                            Expr::from_head(
                                "List",
                                vec![
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(4.0), Symbol("a".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(4.0), Symbol("b".into())]
                                            ),
                                        ]
                                    ),
                                    Expr::from_head(
                                        "List",
                                        vec![
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(4.0), Symbol("c".into())]
                                            ),
                                            Expr::from_head(
                                                "Times",
                                                vec![Number(4.0), Symbol("d".into())]
                                            ),
                                        ]
                                    ),
                                ]
                            )
                        ]
                    )
                ]
            ))
        );
    }
}
