use lexers::Scanner;
use crate::lox_scanner::{Token, TT};
use std::rc::Rc;


// NOTE: do _NOT_ derive Clone, or modify id so it's constant across Expr life
#[derive(Debug)]
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

impl Expr {
    pub fn id(&self) -> usize { self as *const _ as usize }
}

// NOTE: do _NOT_ define Clone because we use address of Expr as symtab id
//       we need that address to stay the same for the Resolver
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    Var(String, Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break(usize),
    Function(String, Vec<String>, Rc<Vec<Stmt>>),
    Return(Expr),
}

pub type ExprResult = Result<Expr, String>;
pub type StmtResult = Result<Stmt, String>;

pub struct LoxParser<I: Iterator<Item=Token>> {
    scanner: Scanner<I>,
    errors: bool,
}

impl<I: Iterator<Item=Token>> LoxParser<I> {
    pub fn new(source: I) -> Self {
        LoxParser{scanner: Scanner::new(source), errors: false}
    }

    fn accept<const N: usize>(&mut self, token_types: [TT; N]) -> Option<Token> {
        let backtrack = self.scanner.checkpoint();
        if let Some(token) = self.scanner.advance() {
            let found = token_types.iter().any(|ttype| match &token.token {
                TT::Str(_) => matches!(ttype, TT::Str(_)),
                TT::Id(_) => matches!(ttype, TT::Id(_)),
                TT::Num(_) => matches!(ttype, TT::Num(_)),
                other => other == ttype
            });
            if found { return Some(self.scanner.lift().last().unwrap()); }
        }
        self.scanner.restore(backtrack);
        None
    }

    fn consume<S: AsRef<str>, const N: usize>(&mut self, token_types: [TT; N],
                              err: S) -> Result<Token, String> {
        match self.accept(token_types) {
            Some(token) => Ok(token),
            None => {
                let bad_token = self.scanner.peek().cloned();
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
                let _ = self.scanner.lift();
                return;
            }
            // alternatively if we've found a keyword we might be starting a
            // statement, try to continue there
            if let Some(peek) = self.scanner.peek() {
                use TT::*;
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
 *                  | returnStmt
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
 *  returnStmt     "= "return" [ expression ] ";" ;
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

impl<I: Iterator<Item=Token>> LoxParser<I> {
    fn assignment(&mut self) -> ExprResult {
        let expr = self.logic_or()?;
        if let Some(token) = self.accept([TT::ASSIGN]) {
            let maybe_bad = Some(token);
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
        while let Some(op) = self.accept([TT::AND]) {
            let rhs = self.equality()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn logic_or(&mut self) -> ExprResult {
        let mut expr = self.logic_and()?;
        while let Some(op) = self.accept([TT::OR]) {
            let rhs = self.logic_and()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ExprResult {
        let mut expr = self.comparison()?;
        while let Some(op) = self.accept([TT::EQ, TT::NE]) {
            let rhs = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn comparison(&mut self ) -> ExprResult {
        let mut expr = self.addition()?;
        while let Some(op) = self.accept([TT::GT, TT::GE, TT::LT, TT::LE]) {
            let rhs = self.addition()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn addition(&mut self) -> ExprResult {
        let mut expr = self.multiplication()?;
        while let Some(op) = self.accept([TT::MINUS, TT::PLUS]) {
            let rhs = self.multiplication()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> ExprResult {
        let mut expr = self.unary()?;
        while let Some(op) = self.accept([TT::SLASH, TT::STAR]) {
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn call_expr(&mut self) -> ExprResult {
        let mut primary = self.primary()?;
        // if there's an OPAREN crawl thread the function Call chain
        while self.accept([TT::OPAREN]).is_some() {
            let mut arguments = Vec::new();
            if self.accept([TT::CPAREN]).is_none() { // 0-arg case
                loop {
                    arguments.push(self.expression()?);
                    if self.accept([TT::COMMA]).is_none() { break; }
                }
                self.consume([TT::CPAREN], "expect ')' after call args")?;
            }
            primary = Expr::Call(Box::new(primary), arguments);
        }
        Ok(primary)
    }

    fn unary(&mut self) -> ExprResult {
        if let Some(op) = self.accept([TT::BANG, TT::MINUS, TT::DOLLAR]) {
            let rhs = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(rhs)));
        }
        self.call_expr()
    }

    fn primary(&mut self) -> ExprResult {
        if let Some(token) = self.accept([TT::FALSE, TT::TRUE]) {
            return Ok(match token.token {
                TT::TRUE => Expr::Bool(true),
                _ => Expr::Bool(false),
            });
        }
        if self.accept([TT::NIL]).is_some() {
            return Ok(Expr::Nil);
        }
        if let Some(token) = self.accept([TT::Num(0.0)]) {
            return Ok(match token.token {
                TT::Num(n) => Expr::Num(n),
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            });
        }
        if let Some(token) = self.accept([TT::Str("".to_string())]) {
            return Ok(match token.token {
                TT::Str(s) => Expr::Str(s),
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            });
        }
        if let Some(token) = self.accept([TT::Id("".to_string())]) {
            return Ok(Expr::Var(token));
        }
        if self.accept([TT::OPAREN]).is_some() {
            let expr = self.expression()?;
            self.consume([TT::CPAREN], "expect ')' after group grouping")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        let bad_token = self.scanner.peek().cloned();
        Err(self.error(bad_token, "expected expression"))
    }

    fn print_stmt(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume([TT::SEMICOLON], "expect ';' after print expr")?;
        Ok(Stmt::Print(expr))
    }

    fn expr_stmt(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume([TT::SEMICOLON], "expect ';' after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn block_stmt(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while let Some(maybe_cbrace) = self.scanner.peek() {
            if maybe_cbrace.token == TT::CBRACE { break; }
            statements.push(self.declaration()?);
        }
        self.consume([TT::CBRACE], "expect '}' after block")?;
        Ok(statements)
    }

    fn if_stmt(&mut self) -> StmtResult {
        self.consume([TT::OPAREN], "expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume([TT::CPAREN], "expect ')' after 'if' condition")?;
        let then_branch = self.statement()?;
        if self.accept([TT::ELSE]).is_some() {
            let else_branch = Some(Box::new(self.statement()?));
            return Ok(Stmt::If(condition, Box::new(then_branch), else_branch));
        }
        Ok(Stmt::If(condition, Box::new(then_branch), None))
    }

    fn while_stmt(&mut self) -> StmtResult {
        self.consume([TT::OPAREN], "expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume([TT::CPAREN], "expect ')' after 'if' condition")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_stmt(&mut self) -> StmtResult {
        self.consume([TT::OPAREN], "expect '(' after 'for'")?;
        let init = if self.accept([TT::SEMICOLON]).is_some() {
            None
        } else if self.accept([TT::VAR]).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };
        // parse loop condition
        let condition = match self.scanner.peek() {
            Some(ref t) if t.token != TT::SEMICOLON => self.expression()?,
            _ => Expr::Bool(true)
        };
        self.consume([TT::SEMICOLON], "expect ';' loop condition")?;
        // parse loop increment
        let increment = match self.scanner.peek() {
            Some(ref t) if t.token != TT::CPAREN => Some(self.expression()?),
            _ => None
        };
        self.consume([TT::CPAREN], "expect ')' after 'for' clause")?;
        // desugar forStmt into WhileStmt
        let body = Stmt::While(condition, Box::new(match increment {
            Some(inc) => Stmt::Block(vec![self.statement()?, Stmt::Expr(inc)]),
            _ => self.statement()?
        }));
        Ok(match init {Some(init) => Stmt::Block(vec![init, body]), _ => body})
    }

    fn break_stmt(&mut self) -> StmtResult {
        let mut scopes = 1;
        if let Some(token) = self.accept([TT::Num(0.0)]) {
            scopes = match token.token {
                TT::Num(n) => n as usize,
                o => panic!("LoxParser Bug! unexpected token: {:?}", o),
            };
        }
        self.consume([TT::SEMICOLON], "expect ';' after 'break'")?;
        Ok(Stmt::Break(scopes))
    }

    fn return_stmt(&mut self) -> StmtResult {
        let expr = match self.scanner.peek() {
            Some(ref t) if t.token != TT::SEMICOLON => self.expression()?,
            _ => Expr::Nil
        };
        self.consume([TT::SEMICOLON], "expect ';' after return value")?;
        Ok(Stmt::Return(expr))
    }

    fn statement(&mut self) -> StmtResult {
        if self.accept([TT::PRINT]).is_some() {
            return self.print_stmt();
        }
        if self.accept([TT::OBRACE]).is_some() {
            return Ok(Stmt::Block(self.block_stmt()?));
        }
        if self.accept([TT::IF]).is_some() {
            return self.if_stmt();
        }
        if self.accept([TT::WHILE]).is_some() {
            return self.while_stmt();
        }
        if self.accept([TT::FOR]).is_some() {
            return self.for_stmt();
        }
        if self.accept([TT::BREAK]).is_some() {
            return self.break_stmt();
        }
        if self.accept([TT::RETURN]).is_some() {
            return self.return_stmt();
        }
        self.expr_stmt()
    }

    fn var_declaration(&mut self) -> StmtResult {
        let name = self.consume(
            [TT::Id("".to_string())], "expect variable name")?;
        let mut init = Expr::Nil;
        if self.accept([TT::ASSIGN]).is_some() {
            init = self.expression()?;
        }
        self.consume([TT::SEMICOLON], "expect ';' after variable decl")?;
        Ok(Stmt::Var(name.lexeme, init))
    }

    fn fun_declaration(&mut self, kind: &str) -> StmtResult {
        let name = self.consume(
            [TT::Id("".to_string())], format!("expect {} name", kind))?;
        self.consume([TT::OPAREN], format!("expect '(' after {}", kind))?;
        let mut params = Vec::new();
        if self.accept([TT::CPAREN]).is_none() {
            loop {
                let parameter = self.consume(
                    [TT::Id("".to_string())], "expect parameter name")?;
                params.push(parameter.lexeme);
                if self.accept([TT::COMMA]).is_none() { break; }
            }
            self.consume([TT::CPAREN], "expect ')' after parameters")?;
        }
        self.consume(
            [TT::OBRACE], format!("expect '{{' before {} body ", kind))?;
        Ok(Stmt::Function(name.lexeme, params, Rc::new(self.block_stmt()?)))
    }

    fn declaration(&mut self) -> StmtResult {
        if self.accept([TT::VAR]).is_some() {
            return self.var_declaration();
        }
        if self.accept([TT::FUN]).is_some() {
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
        match errors.is_empty() {
            false => Err(errors),
            true => Ok(statements)
        }
    }
}
