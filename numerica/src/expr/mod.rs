mod arithmetic;
mod distribution;
mod find_root;
mod listops;
mod replace_all;
mod sum;
mod table;
mod transcendental;

pub use distribution::is_stochastic;

use core::fmt;
use std::rc::Rc;

use crate::context::Context;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Head(Box<Expr>, Vec<Expr>),
    Symbol(String),
    Number(f64),
    Bool(bool),
    String(String),
    Distribution(Rc<distribution::Distr>),
    Function(Vec<String>, Box<Expr>),
    // DateTime(DateTime<Utc>),
    // Matrix(Matrix),
    // Quantity(f64, Dimension),
    // Complex(f64, f64),
    // List(Vec<Expr>), // should this just be a primitive too ?
}

impl Expr {
    pub fn from_head(head: &str, args: Vec<Expr>) -> Self {
        Expr::Head(Box::new(Expr::Symbol(head.into())), args)
    }
}

// Lowest number is highest precedence
fn op_print_precedence(e: &Expr) -> Option<usize> {
    if let Expr::Head(h, _) = e {
        if let Expr::Symbol(ref symbol) = **h {
            return match symbol.as_ref() {
                "Power" => Some(50),
                "Unsure" => Some(55),
                "Divide" => Some(60),
                "Times" => Some(60),
                "Plus" => Some(70),
                "Minus" => Some(70),
                _ => None,
            };
        }
    }
    None
}

fn join_args(e: &Expr, sep: &str) -> String {
    let parent_p = op_print_precedence(e);
    let Expr::Head(_, args) = e else {
        panic!("BUG: Tried to join_args for non Expr: {:?}", e);
    };
    args.iter()
        .map(|a| {
            if parent_p.is_some() && parent_p < op_print_precedence(a) {
                format!("({})", a)
            } else {
                a.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Symbol(s) => write!(f, "{}", s),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Head(h, _) if matches!(**h, Expr::Symbol(_)) => {
                let Expr::Symbol(head_name) = h.as_ref() else {
                    unreachable!()
                };
                match head_name.as_ref() {
                    "Plus" => write!(f, "{}", join_args(self, " + ")),
                    "Unsure" => write!(f, "{}", join_args(self, "~")),
                    "Times" => write!(f, "{}", join_args(self, " * ")),
                    "Minus" => write!(f, "{}", join_args(self, " - ")),
                    "Divide" => write!(f, "{}", join_args(self, " / ")),
                    "Power" => write!(f, "{}", join_args(self, " ^ ")),
                    "List" => write!(f, "{{{}}}", join_args(self, ", ")),
                    _ => write!(f, "{}[{}]", head_name, join_args(self, ", ")),
                }
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

// Evaluate an expression reducing it as far as possible recurring the AST.
pub fn evaluate(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    match expr {
        Expr::Head(head, args) => {
            // Evaluate the head which can be an expression itself.
            let head = evaluate(*head, ctx)?;
            // Evaluate arguments (unless holding)
            let args = match head {
                Expr::Symbol(ref h) if h == "SetDelayed" => args, // skip eval of LHS, RHS
                Expr::Symbol(ref h) if h == "Set" => {
                    let [lhs, rhs]: [Expr; 2] = args
                        .try_into()
                        .map_err(|e| format!("Set must have 2 arguments. {:?}", e))?;
                    // skip eval of LHS, but RHS should be evaluated
                    vec![lhs, evaluate(rhs, ctx)?]
                }
                Expr::Symbol(ref h) if h == "Rule" => {
                    let [lhs, rhs]: [Expr; 2] = args
                        .try_into()
                        .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
                    // Rule RHS is evaluated before being stored (as opposed to RuleDelayed)
                    vec![lhs, evaluate(rhs, ctx)?]
                }
                // Mathematica's Function has HoldAll attribute. Body is evaluated at runtime.
                Expr::Symbol(ref h) if h == "Function" => args,
                Expr::Symbol(ref h) if h == "Hold" => args,
                _ => args
                    .into_iter()
                    .map(|arg| evaluate(arg, ctx))
                    .collect::<Result<Vec<_>, _>>()?,
            };
            // Apply the evaluated head to the evaluated arguments
            apply(head, args, ctx)
        }
        Expr::Symbol(ref sym) => match ctx.get(sym) {
            Some(value) => evaluate(value, ctx),
            None => Ok(expr),
        },
        _ => Ok(expr), // Primitive values don't require further evaluation.
    }
}

// Execute callable application logic. At this point head has been pre-evaluated and
// shouldn't need further evaluation (except maybe for some special case like ReplaceAll).
pub(crate) fn apply(head: Expr, args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    match head {
        Expr::Symbol(ref head_sym) => match head_sym.as_str() {
            "Hold" | "List" => Ok(Expr::Head(Box::new(head), args)),
            "Rule" | "RuleDelayed" => Ok(Expr::Head(Box::new(head), args)),
            "ReplaceAll" => {
                // ReplaceAll execution will do the rewrites.
                let replaced = replace_all::eval_replace_all(args)?;
                // However the result may furhter need an evaluation.
                // Eg.  Sin[x] /. x -> 3  (* gives Sin[3] *) which we want a number for
                // NOTE: this could live in evaluate. Calling it here is an exceptional case !
                evaluate(replaced, ctx)
            }
            "Plus" => arithmetic::eval_plus(args),
            "Times" => arithmetic::eval_times(args),
            "Minus" => arithmetic::eval_minus(args),
            "Power" => arithmetic::eval_power(args),
            "Divide" => arithmetic::eval_divide(args),
            "FindRoot" => find_root::eval_find_root(args),
            "Sum" => sum::eval_sum(args, ctx),
            "SetDelayed" | "Set" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Set(Delayed) must have 2 arguments. {:?}", e))?;
                let Expr::Symbol(lhs) = lhs else {
                    return Err(format!("Set(Delayed) lhs must be a symbol. {:?}", lhs));
                };
                ctx.set(lhs, rhs.clone());
                Ok(rhs)
            }
            "Gamma" => transcendental::eval_gamma(args),
            "NormalDist" => distribution::eval_normal_dist(args),
            "BetaDist" => distribution::eval_beta_dist(args),
            "PoissonDist" => distribution::eval_poisson_dist(args),
            "Sin" => transcendental::eval_sin(args),
            "Cos" => transcendental::eval_cos(args),
            "Exp" => transcendental::eval_exp(args),
            "Abs" => transcendental::eval_abs(args),
            "Table" => table::eval_table(args, ctx),
            "Function" => {
                let [params, body]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Function must have params and body. {:?}", e))?;
                let params = match params {
                    Expr::Symbol(sym) => Ok(vec![sym]),
                    Expr::Head(h, syms) if *h == Expr::Symbol("List".into()) => syms
                        .into_iter()
                        .map(|s| match s {
                            Expr::Symbol(sym) => Ok(sym),
                            _ => Err(format!("Function params must be symbols: {}", s)),
                        })
                        .collect(),
                    o => Err(format!("Function params must be symbols: {}", o)),
                }?;
                Ok(Expr::Function(params, Box::new(body)))
            }
            "Unsure" => distribution::eval_unsure(args),
            "Sample" => distribution::eval_sample(args, ctx),
            "Histogram" => distribution::eval_histogram(args, ctx),
            "Outer" => listops::eval_outer(args, ctx),
            // Return verbatim expression by default
            _ => Ok(Expr::Head(Box::new(head), args)),
        },
        Expr::Distribution(d) => Ok(Expr::Number(d.sample())),
        Expr::Function(params, body) => {
            if params.len() != args.len() {
                return Err(format!(
                    "Function expected {} args but got {}",
                    params.len(),
                    args.len()
                ));
            }
            // Create a new context scoped for the function call
            let mut f_ctx = ctx.extend();
            for (p, a) in params.into_iter().zip(args) {
                f_ctx.set(p, a);
            }
            // Body hasn't been evaluated yet. Now that we've got
            // values for all parameters, we can evaluate the body.
            evaluate(*body, &mut f_ctx)
        }
        _ => Err(format!("Non-callable head {}", head)),
    }
}
