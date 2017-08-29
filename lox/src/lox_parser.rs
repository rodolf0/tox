#![deny(warnings)]

extern crate lexers;
use self::lexers::Scanner;

use lox_scanner::{Token, TT};


#[derive(Clone,Debug)]
pub enum Expr {
    Logical(Box<Expr>, Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Grouping(Box<Expr>),
    Var(Token),
    Assign(Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Clone)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    Var(String, Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break(usize),
    Function(String, Vec<String>, Vec<Stmt>),
}

pub type ExprResult = Result<Expr, String>;
pub type StmtResult = Result<Stmt, String>;

pub struct LoxParser {
    scanner: Scanner<Token>,
    errors: bool,
}

impl LoxParser {
    pub fn new(scanner: Scanner<Token>) -> Self {
        LoxParser{scanner: scanner, errors: false}
    }

    fn accept(&mut self, token_types: Vec<TT>) -> bool {
        let backtrack = self.scanner.pos();
        if let Some(token) = self.scanner.next() {
            let found = token_types.iter().any(|ttype| match &token.token {
                &TT::Str(_) => match ttype { &TT::Str(_) => true, _ => false },
                &TT::Id(_) => match ttype { &TT::Id(_) => true, _ => false },
                &TT::Num(_) => match ttype { &TT::Num(_) => true, _ => false },
                other => other == ttype
            });
            if found { return true; }
        }
        self.scanner.set_pos(backtrack);
        false
    }

    fn consume<S: AsRef<str>>(&mut self, token_types: Vec<TT>,
                              err: S) -> Result<Token, String> {
        match self.accept(token_types) {
            true => Ok(self.scanner.extract().swap_remove(0)),
            false => {
                let bad_token = self.scanner.peek();
                Err(self.error(bad_token, err))
            }
        }
    }

    fn error<S: AsRef<str>>(&mut self, token: Option<Token>, msg: S) -> String {
        self.errors = true;
        match token {
            Some(t) => format!("LoxParser error: {:?} at line {}, {}",
                               t.lexeme, t.line, msg.as_ref()),
            _ => format!("LoxParser error: EOF, {}", msg.as_ref()),
        }
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.scanner.next() {
            // if we hit a semicolon we're probably about to start a statement
            // we maybe inside a `for` clause, too bad, we're already panic'ing
            if token.token == TT::SEMICOLON {
                return self.scanner.ignore();
            }
            // alternatively if we've found a keyword we might be starting a
            // statement, try to continue there
            if let Some(peek) = self.scanner.peek() {
                use self::TT::*;
                match peek.token {
                    CLASS | FUN | VAR | FOR | IF |
                    WHILE | PRINT | RETURN | BREAK
                    => return, _ => ()
                }
            }
        }
    }
}


/* Grammar:
 *
 *  program        := { declaration } EOF ;
 *
 *  declaration    := varDecl
 *                  | funDecl
 *                  | statement ;
 *
 *  funDecl        := "fun" function ;
 *  function       := IDENTIFIER "(" [ parameters ] ")" block ;
 *  parameters     := IDENTIFIER { "," IDENTIFIER } ;
 *
 *  varDecl        := "var" IDENTIFIER [ "=" expression ] ";" ;
 *
 *  statement      := exprStmt
 *                  | ifStmt
 *                  | printStmt
 *                  | whileStmt
 *                  | forStmt
 *                  | breakStmt
 *                  | block ;
 *
 *  exprStmt       := expression ";" ;
 *  ifStmt         := "if" "(" expression ")" statement [ "else" statement ] ;
 *  printStmt      := "print" expression ";" ;
 *  whileStmt      := "while" "(" expression ")" statement ;
 *  forStmt        := "for" "(" varDecl | exprStmt | ";"
 *                            { expression } ";"
 *                            { expression } ")" statement ;
 *  breakStmt      := "break" [ NUMBER ] ";" ;
 *  block          := "{" { declaration } "}" ;
 *
 *  expression     := assignment ;
 *  assignment     := identifier "=" assignment
 *                  | logic_or ;
 *  logic_or       := logic_and { "or" logic_and } ;
 *  logic_and      := equality { "and" equality } ;
 *  equality       := comparison { ( "!=" | "==" ) comparison } ;
 *  comparison     := addition { ( ">" | ">=" | "<" | "<=" ) addition } ;
 *  addition       := multiplication { ( "-" | "+" ) multiplication } ;
 *  multiplication := unary { ( "/" | "*" ) unary } ;
 *  unary          := ( "!" | "-" | "$" ) unary
 *                  | call_expr;
 *  call_expr      := primary { "(" [ arguments ] ")" } ; // hi precedence op()
 *  arguments      := expression { "," expression } ;
 *  primary        := NUMBER | STRING | "false" | "true" | "nil"
 *                  | "(" expression ")"
 *                  | IDENTIFIER ;
 */

impl LoxParser {
    fn assignment(&mut self) -> ExprResult {
        let expr = self.logic_or()?;
        if self.accept(vec![TT::ASSIGN]) {
            let maybe_bad = Some(self.scanner.extract().swap_remove(0));
            // recursively parse right-hand-side
            let value = self.assignment()?;
            return match expr {
                // assign to variable, later other lhs possible
                Expr::Var(name) => Ok(Expr::Assign(name, Box::new(value))),
                _ => Err(self.error(maybe_bad, "invalid assignment target"))
            };
        }
        Ok(expr)
    }

    fn expression(&mut self) -> ExprResult {
        self.assignment()
    }

    fn logic_and(&mut self) -> ExprResult {
        let mut expr = self.equality()?;
        while self.accept(vec![TT::AND]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.equality()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn logic_or(&mut self) -> ExprResult {
        let mut expr = self.logic_and()?;
        while self.accept(vec![TT::OR]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.logic_and()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ExprResult {
        let mut expr = self.comparison()?;
        while self.accept(vec![TT::EQ, TT::NE]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn comparison(&mut self ) -> ExprResult {
        let mut expr = self.addition()?;
        while self.accept(vec![TT::GT, TT::GE, TT::LT, TT::LE]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.addition()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn addition(&mut self) -> ExprResult {
        let mut expr = self.multiplication()?;
        while self.accept(vec![TT::MINUS, TT::PLUS]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.multiplication()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> ExprResult {
        let mut expr = self.unary()?;
        while self.accept(vec![TT::SLASH, TT::STAR]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn call_expr(&mut self) -> ExprResult {
        let mut primary = self.primary()?;
        // if there's an OPAREN crawl thread the function Call chain
        while self.accept(vec![TT::OPAREN]) {
            self.scanner.ignore(); // skip oparen
            let mut arguments = Vec::new();
            if !self.accept(vec![TT::CPAREN]) { // 0-arg case
                loop {
                    arguments.push(self.expression()?);
                    if !self.accept(vec![TT::COMMA]) { break; }
                    self.scanner.ignore(); // skip comma
                }
                self.consume(vec![TT::CPAREN], "expect ')' after call args")?;
            }
            self.scanner.ignore(); // skip cparen if accepted
            primary = Expr::Call(Box::new(primary), arguments);
        }
        Ok(primary)
    }

    fn unary(&mut self) -> ExprResult {
        if self.accept(vec![TT::BANG, TT::MINUS, TT::DOLLAR]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(rhs)));
        }
        self.call_expr()
    }

    fn primary(&mut self) -> ExprResult {
        if self.accept(vec![TT::FALSE, TT::TRUE]) {
            return Ok(match self.scanner.extract().swap_remove(0).token {
                TT::TRUE => Expr::Bool(true),
                _ => Expr::Bool(false),
            });
        }
        if self.accept(vec![TT::NIL]) {
            self.scanner.ignore();
            return Ok(Expr::Nil);
        }
        if self.accept(vec![TT::Num(0.0)]) {
            return Ok(match self.scanner.extract().swap_remove(0).token {
                TT::Num(n) => Expr::Num(n),
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            });
        }
        if self.accept(vec![TT::Str("".to_string())]) {
            return Ok(match self.scanner.extract().swap_remove(0).token {
                TT::Str(s) => Expr::Str(s),
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            });
        }
        if self.accept(vec![TT::Id("".to_string())]) {
            return Ok(Expr::Var(self.scanner.extract().swap_remove(0)));
        }
        if self.accept(vec![TT::OPAREN]) {
            self.scanner.ignore(); // skip OPAREN
            let expr = self.expression()?;
            self.consume(vec![TT::CPAREN], "expect ')' after group grouping")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        let bad_token = self.scanner.peek();
        Err(self.error(bad_token, "expected expression"))
    }

    fn print_stmt(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume(vec![TT::SEMICOLON], "expect ';' after print expr")?;
        Ok(Stmt::Print(expr))
    }

    fn expr_stmt(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume(vec![TT::SEMICOLON], "expect ';' after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn block_stmt(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while let Some(maybe_cbrace) = self.scanner.peek() {
            if maybe_cbrace.token == TT::CBRACE { break; }
            statements.push(self.declaration()?);
        }
        self.consume(vec![TT::CBRACE], "expect '}' after block")?;
        Ok(statements)
    }

    fn if_stmt(&mut self) -> StmtResult {
        self.consume(vec![TT::OPAREN], "expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(vec![TT::CPAREN], "expect ')' after 'if' condition")?;
        let then_branch = self.statement()?;
        if self.accept(vec![TT::ELSE]) {
            self.scanner.ignore(); // skip else
            let else_branch = Some(Box::new(self.statement()?));
            return Ok(Stmt::If(condition, Box::new(then_branch), else_branch));
        }
        Ok(Stmt::If(condition, Box::new(then_branch), None))
    }

    fn while_stmt(&mut self) -> StmtResult {
        self.consume(vec![TT::OPAREN], "expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(vec![TT::CPAREN], "expect ')' after 'if' condition")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_stmt(&mut self) -> StmtResult {
        self.consume(vec![TT::OPAREN], "expect '(' after 'for'")?;
        let init = if self.accept(vec![TT::SEMICOLON]) {
            self.scanner.ignore(); // skip ';'
            None
        } else if self.accept(vec![TT::VAR]) {
            self.scanner.ignore(); // skip var
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };
        // parse loop condition
        let condition = match self.scanner.peek() {
            Some(ref t) if t.token != TT::SEMICOLON => self.expression()?,
            _ => Expr::Bool(true)
        };
        self.consume(vec![TT::SEMICOLON], "expect ';' loop condition")?;
        // parse loop increment
        let increment = match self.scanner.peek() {
            Some(ref t) if t.token != TT::CPAREN => Some(self.expression()?),
            _ => None
        };
        self.consume(vec![TT::CPAREN], "expect ')' after 'for' clause")?;
        // desugar forStmt into WhileStmt
        let body = Stmt::While(condition, Box::new(match increment {
            Some(inc) => Stmt::Block(vec![self.statement()?, Stmt::Expr(inc)]),
            _ => self.statement()?
        }));
        Ok(match init {Some(init) => Stmt::Block(vec![init, body]), _ => body})
    }

    fn break_stmt(&mut self) -> StmtResult {
        let mut scopes = 1;
        if self.accept(vec![TT::Num(0.0)]) {
            scopes = match self.scanner.extract().swap_remove(0).token {
                TT::Num(n) => n as usize,
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            };
        }
        self.consume(vec![TT::SEMICOLON], "expect ';' 'break'")?;
        Ok(Stmt::Break(scopes))
    }

    fn statement(&mut self) -> StmtResult {
        if self.accept(vec![TT::PRINT]) {
            self.scanner.ignore(); // skip print
            return self.print_stmt();
        }
        if self.accept(vec![TT::OBRACE]) {
            self.scanner.ignore(); // skip obrace
            return Ok(Stmt::Block(self.block_stmt()?));
        }
        if self.accept(vec![TT::IF]) {
            self.scanner.ignore(); // skip if
            return self.if_stmt();
        }
        if self.accept(vec![TT::WHILE]) {
            self.scanner.ignore(); // skip while
            return self.while_stmt();
        }
        if self.accept(vec![TT::FOR]) {
            self.scanner.ignore(); // skip for
            return self.for_stmt();
        }
        if self.accept(vec![TT::BREAK]) {
            self.scanner.ignore(); // skip break
            return self.break_stmt();
        }
        self.expr_stmt()
    }

    fn var_declaration(&mut self) -> StmtResult {
        let name = self.consume(
            vec![TT::Id("".to_string())], "expect variable name")?;
        let mut init = Expr::Nil;
        if self.accept(vec![TT::ASSIGN]) {
            self.scanner.ignore(); // skip assign
            init = self.expression()?;
        }
        self.consume(vec![TT::SEMICOLON], "expect ';' after variable decl")?;
        Ok(Stmt::Var(name.lexeme, init))
    }

    fn fun_declaration(&mut self, kind: &str) -> StmtResult {
        let name = self.consume(
            vec![TT::Id("".to_string())], format!("expect {} name", kind))?;
        self.consume(vec![TT::OPAREN], format!("expect '(' after {}", kind))?;
        let mut params = Vec::new();
        if !self.accept(vec![TT::CPAREN]) {
            loop {
                let parameter = self.consume(
                    vec![TT::Id("".to_string())], "expect parameter name")?;
                params.push(parameter.lexeme);
                if !self.accept(vec![TT::COMMA]) { break; }
                self.scanner.ignore(); // skip comma
            }
            self.consume(vec![TT::CPAREN], "expect ')' after parameters")?;
        }
        self.consume(
            vec![TT::OBRACE], format!("expect '{{' before {} body ", kind))?;
        Ok(Stmt::Function(name.lexeme, params, self.block_stmt()?))
    }

    fn declaration(&mut self) -> StmtResult {
        if self.accept(vec![TT::VAR]) {
            self.scanner.ignore(); // skip var
            return self.var_declaration();
        }
        if self.accept(vec![TT::FUN]) {
            self.scanner.ignore(); // skip fun
            return self.fun_declaration("function");
        }
        self.statement()
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<String>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while self.scanner.peek().is_some() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => { errors.push(err); self.synchronize(); }
            }
        }
        match errors.len() > 0 {
            true => Err(errors),
            false => Ok(statements)
        }
    }
}
