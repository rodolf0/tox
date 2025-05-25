use super::Expr;
use crate::context::Context;
use std::fs;

pub(crate) fn eval_get(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [file_path]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Get expects a file name. {:?}", e))?;

    let file_path = match file_path {
        Expr::String(file_path) => Ok(file_path),
        o => Err(format!("Expected string file path. {:?}", o)),
    }?;

    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;

    crate::evaluate(crate::parser()?(&content)?, ctx)
}
