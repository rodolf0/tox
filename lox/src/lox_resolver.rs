#![deny(warnings)]

use crate::lox_parser::{Expr, Stmt};
use crate::lox_interpreter::LoxInterpreter;
use std::collections::HashMap;

type ResolveResult = Result<(), String>;

pub struct Resolver<'a> {
    interpreter: &'a mut LoxInterpreter,
    // tracks if variable name is defined or just declared
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interp: &'a mut LoxInterpreter) -> Resolver {
        Resolver{interpreter: interp, scopes: Vec::new()}
    }

    pub fn resolve(&mut self, stmts: &[Stmt]) -> ResolveResult {
        stmts.iter().map(|stmt| self.resolve_stmt(stmt))
             .skip_while(|stmt| stmt.is_ok())
             .next()
             .unwrap_or(Ok(()))
    }

    fn begin_scope(&mut self) { self.scopes.push(HashMap::new()); }

    fn end_scope(&mut self) { self.scopes.pop(); }

    fn declare(&mut self, token: String) -> ResolveResult {
        use std::collections::hash_map::Entry::*;
        if let Some(scope) = self.scopes.last_mut() {
            match scope.entry(token.clone()) {
                Occupied(_) => return
                    Err(format!("Var {} already declared in scope", token)),
                Vacant(spot) => { spot.insert(false); }
            }
        }
        Ok(())
    }

    fn define(&mut self, token: String) -> ResolveResult {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(token, true);
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: &str) -> ResolveResult {
        // find the scope that contains the name
        let scope = self.scopes.iter().rev()
            .enumerate()
            .skip_while(|&(_, scope)| !scope.contains_key(name))
            .next();
        // bind the interpreter's reference to that scope
        if let Some((idx, _)) = scope {
            self.interpreter.resolve(expr.id(), idx);
        }
        Ok(())
    }

    fn resolve_function(&mut self, params: &[String],
                        body: &[Stmt]) -> ResolveResult {
        self.begin_scope();
        for param in params {
            self.declare(param.clone())?;
            self.define(param.clone())?;
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolveResult {
        match expr {
            &Expr::Logical(ref left, _, ref right) => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            },
            &Expr::Binary(ref left, _, ref right) => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            },
            &Expr::Unary(_, ref unary) => self.resolve_expr(unary),
            &Expr::Nil | &Expr::Bool(_) |
            &Expr::Num(_) | &Expr::Str(_) => Ok(()),
            &Expr::Grouping(ref gexpr) => self.resolve_expr(gexpr),
            &Expr::Var(ref token) => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&token.lexeme) == Some(&false) {
                        return Err(format!(
                            "Can't read var in initializer {:?}", token));
                    }
                }
                self.resolve_local(expr, &token.lexeme)
            },
            &Expr::Assign(ref token, ref asigex) => {
                self.resolve_expr(asigex)?;
                self.resolve_local(expr, &token.lexeme)
            },
            &Expr::Call(ref callee, ref args) => {
                self.resolve_expr(callee)?;
                args.iter().map(|arg| self.resolve_expr(arg))
                    .skip_while(|arg| arg.is_ok())
                    .next()
                    .unwrap_or(Ok(()))
            },
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt)  -> ResolveResult {
        match stmt {
            Stmt::Print(ref expr) => self.resolve_expr(expr),
            Stmt::Expr(ref expr) => self.resolve_expr(expr),
            Stmt::Var(ref name, ref init) => {
                // split binding in declare/define to disallow self reference
                self.declare(name.clone())?;
                match init {
                    Expr::Nil => (),
                    _ => self.resolve_expr(init)?,
                }
                self.define(name.clone())
            },
            Stmt::Block(ref stmts) => {
                self.begin_scope();
                self.resolve(stmts)?;
                self.end_scope();
                Ok(())
            },
            Stmt::If(ref cond, ref then, ref elseb) => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(then)?;
                if let Some(ref elseb) = elseb {
                    self.resolve_stmt(elseb)?;
                }
                Ok(())
            },
            Stmt::While(ref cond, ref body) => {
                self.resolve_expr(cond)?;
                self.resolve_stmt(body)
            },
            Stmt::Break(_) => Ok(()),
            Stmt::Function(ref name, ref parameters, ref body) => {
                self.declare(name.clone())?;
                self.define(name.clone())?;
                self.resolve_function(parameters, body)
            },
            Stmt::Return(ref expr) => self.resolve_expr(expr),
        }
    }
}
