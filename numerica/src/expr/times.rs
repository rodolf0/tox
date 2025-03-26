use super::{Expr, eval_with_ctx};
use crate::context::Context;

fn distribute_op(lhs: Expr, rhs: Expr, op: &str, over: &str) -> Expr {
    match lhs {
        Expr::Expr(h, lhs) if h == over => match rhs {
            Expr::Expr(h, rhs) if h == over => Expr::Expr(
                over.to_string(),
                lhs.iter()
                    .flat_map(|lhsi| {
                        rhs.iter()
                            .map(|rhsi| distribute_op(lhsi.clone(), rhsi.clone(), op, over))
                    })
                    .collect(),
            ),
            rhs => Expr::Expr(
                over.to_string(),
                lhs.into_iter()
                    .map(|lhsi| distribute_op(lhsi, rhs.clone(), op, over))
                    .collect(),
            ),
        },
        lhs => match rhs {
            Expr::Expr(h, rhs) if h == over => Expr::Expr(
                over.to_string(),
                rhs.into_iter()
                    .map(|rhsi| distribute_op(lhs.clone(), rhsi, op, over))
                    .collect(),
            ),
            rhs => Expr::Expr(op.to_string(), vec![lhs, rhs]),
        },
    }
}

fn flatten(expr: Expr, op: &str) -> Expr {
    match expr {
        Expr::Expr(head, args) if head == op => Expr::Expr(
            head,
            args.into_iter()
                .flat_map(|ai| match ai {
                    Expr::Expr(h, a) if h == op => {
                        a.into_iter().map(|aj| flatten(aj, op)).collect()
                    }
                    other => vec![flatten(other, op)],
                })
                .collect(),
        ),
        Expr::Expr(head, args) => {
            Expr::Expr(head, args.into_iter().map(|ai| flatten(ai, op)).collect())
        }
        other => other,
    }
}

pub fn eval_times(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    // distribute over List
    // 3 * {a ,b} => {3 * a, 3 * b}
    // {a, b} * 3 => {3 * a, 3 * b}
    // {a, b} * {x, y} => {a * x, a * y, b * x, b * y}
    // 3 * x => 3 * x
    // 3 * x * {a, b} => {3 * x * a, 3 * x * b}
    // 3 * {4, 5} => {12, 15}

    // Distribute Times over List
    let mut new_args = vec![eval_with_ctx(args.remove(0), ctx)?];
    for arg in args {
        let rhs = eval_with_ctx(arg, ctx)?;
        new_args = new_args
            .into_iter()
            .map(|lhsi| flatten(distribute_op(lhsi, rhs.clone(), "Times", "List"), "Times"))
            .collect();
    }
    // Peel off wrapping Times result of distributing Times over List to avoid inf recurse
    new_args = new_args
        .into_iter()
        .flat_map(|expr| match expr {
            Expr::Expr(h, a) if h == "Times" => a,
            o => vec![o],
        })
        .collect();
    // Run mmultiplication
    let mut numeric: Option<f64> = None;
    let mut new_args2 = Vec::new();
    for arg in new_args {
        match eval_with_ctx(arg, ctx)? {
            Expr::Number(n) => *numeric.get_or_insert(1.0) *= n,
            o => new_args2.push(o),
        }
    }
    if numeric.is_some_and(|n| n != 1.0) || new_args2.len() == 0 {
        new_args2.insert(0, Expr::Number(numeric.unwrap()));
    }
    if new_args2.len() == 1 {
        Ok(new_args2.swap_remove(0))
    } else {
        Ok(Expr::Expr("Times".to_string(), new_args2))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;

    #[test]
    fn test_simple_multiplication() {
        // test single arg
        let args = vec![Expr::Number(3.0)];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(result, Expr::Number(3.0));

        // test numerics
        let args = vec![Expr::Number(3.0), Expr::Number(4.0)];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(result, Expr::Number(12.0));

        // test identity
        let args = vec![Expr::Number(1.0), Expr::Symbol("x".to_string())];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(result, Expr::Symbol("x".to_string()));

        // test rhs multiplication
        let args = vec![Expr::Symbol("x".to_string()), Expr::Number(1.0)];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(result, Expr::Symbol("x".to_string()));

        // test symbol multiplication
        let args = vec![Expr::Symbol("x".to_string()), Expr::Symbol("y".to_string())];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr::Expr(
                "Times".to_string(),
                vec![Expr::Symbol("x".to_string()), Expr::Symbol("y".to_string())]
            )
        );
    }

    #[test]
    fn test_distribute_over_list() {
        use crate::expr::Expr::*;
        // distribution of number on LHS
        let args = vec![
            Number(2.0),
            Expr("List".to_string(), vec![Number(3.0), Number(4.0)]),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr("List".to_string(), vec![Number(6.0), Number(8.0)])
        );

        // distribution of sym on LHS
        let args = vec![
            Symbol("x".to_string()),
            Expr("List".to_string(), vec![Number(3.0), Number(4.0)]),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![Number(3.0), Symbol("x".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Number(4.0), Symbol("x".to_string())]
                    )
                ]
            )
        );

        // distribution of sym on RHS
        let args = vec![
            Expr("List".to_string(), vec![Number(3.0), Number(4.0)]),
            Symbol("x".to_string()),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![Number(3.0), Symbol("x".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Number(4.0), Symbol("x".to_string())]
                    )
                ]
            )
        );
    }

    #[test]
    fn test_list_times_list() {
        use crate::expr::Expr::*;

        // {1, 2} * {x, y}
        let args = vec![
            Expr("List".to_string(), vec![Number(1.0), Number(2.0)]),
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Symbol("x".to_string()),
                    Symbol("y".to_string()),
                    Expr(
                        "Times".to_string(),
                        vec![Number(2.0), Symbol("x".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Number(2.0), Symbol("y".to_string())]
                    ),
                ],
            )
        );

        // {x, y} * {1, 2}
        let args = vec![
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
            Expr("List".to_string(), vec![Number(1.0), Number(2.0)]),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Symbol("x".to_string()),
                    Expr(
                        "Times".to_string(),
                        vec![Number(2.0), Symbol("x".to_string())]
                    ),
                    Symbol("y".to_string()),
                    Expr(
                        "Times".to_string(),
                        vec![Number(2.0), Symbol("y".to_string())]
                    ),
                ],
            )
        );

        // Check numeric lists
        let args = vec![
            Expr("List".to_string(), vec![Number(1.0), Number(2.0)]),
            Expr("List".to_string(), vec![Number(3.0), Number(4.0)]),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![Number(3.0), Number(4.0), Number(6.0), Number(8.0)]
            )
        );

        // {x, y} * {w, z}
        let args = vec![
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
            Expr(
                "List".to_string(),
                vec![Symbol("w".to_string()), Symbol("z".to_string())],
            ),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("x".to_string()), Symbol("w".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("x".to_string()), Symbol("z".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("y".to_string()), Symbol("w".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("y".to_string()), Symbol("z".to_string())]
                    ),
                ],
            )
        );
    }

    #[test]
    fn test_complex_distribution() {
        use crate::expr::Expr::*;

        // {x, y} * {w, z} * 3
        let args = vec![
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
            Expr(
                "List".to_string(),
                vec![Symbol("w".to_string()), Symbol("z".to_string())],
            ),
            Number(3.0),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("x".to_string()),
                            Symbol("w".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("x".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("y".to_string()),
                            Symbol("w".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("y".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                ],
            )
        );

        // 4 * {x, y} * {6, z} * 3
        let args = vec![
            Number(4.0),
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
            Expr(
                "List".to_string(),
                vec![Number(6.0), Symbol("z".to_string())],
            ),
            Number(3.0),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![Number(72.0), Symbol("x".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(12.0),
                            Symbol("x".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Number(72.0), Symbol("y".to_string()),]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(12.0),
                            Symbol("y".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                ],
            )
        );

        // a * {x, y} * {w, z} * 3
        let args = vec![
            Symbol("a".to_string()),
            Expr(
                "List".to_string(),
                vec![Symbol("x".to_string()), Symbol("y".to_string())],
            ),
            Expr(
                "List".to_string(),
                vec![Symbol("w".to_string()), Symbol("z".to_string())],
            ),
            Number(3.0),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("a".to_string()),
                            Symbol("x".to_string()),
                            Symbol("w".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("a".to_string()),
                            Symbol("x".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("a".to_string()),
                            Symbol("y".to_string()),
                            Symbol("w".to_string())
                        ]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![
                            Number(3.0),
                            Symbol("a".to_string()),
                            Symbol("y".to_string()),
                            Symbol("z".to_string())
                        ]
                    ),
                ],
            )
        );
    }

    #[test]
    fn test_nested_distribution() {
        use crate::expr::Expr::*;

        // {x, {a, b}} * {w, z}
        let args = vec![
            Expr(
                "List".to_string(),
                vec![
                    Symbol("x".to_string()),
                    Expr(
                        "List".to_string(),
                        vec![Symbol("a".to_string()), Symbol("b".to_string())],
                    ),
                ],
            ),
            Expr(
                "List".to_string(),
                vec![Symbol("w".to_string()), Symbol("z".to_string())],
            ),
        ];
        let result = eval_times(args, &mut Context::new()).unwrap();
        assert_eq!(
            result,
            Expr(
                "List".to_string(),
                vec![
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("x".to_string()), Symbol("w".to_string())]
                    ),
                    Expr(
                        "Times".to_string(),
                        vec![Symbol("x".to_string()), Symbol("z".to_string())]
                    ),
                    Expr(
                        "List".to_string(),
                        vec![
                            Expr(
                                "Times".to_string(),
                                vec![Symbol("a".to_string()), Symbol("w".to_string())]
                            ),
                            Expr(
                                "Times".to_string(),
                                vec![Symbol("b".to_string()), Symbol("w".to_string())]
                            ),
                        ],
                    ),
                    Expr(
                        "List".to_string(),
                        vec![
                            Expr(
                                "Times".to_string(),
                                vec![Symbol("a".to_string()), Symbol("z".to_string())]
                            ),
                            Expr(
                                "Times".to_string(),
                                vec![Symbol("b".to_string()), Symbol("z".to_string())]
                            ),
                        ],
                    )
                ],
            )
        );
    }
}
