#![deny(warnings)]

extern crate lexers;
use self::lexers::Scanner;

use lox_scanner::{Token, TT};


#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Bool(bool),
    Nil,
    Num(f64),
    Str(String),
    Grouping(Box<Expr>),
}

pub struct LoxParser {
    scanner: Scanner<Token>,
    errors: bool,
}

pub type ExprResult = Result<Expr, String>;

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
            if found {
                return true;
            }
        }
        self.scanner.set_pos(backtrack);
        false
    }

    fn consume<S: AsRef<str>>(&mut self, token_types: Vec<TT>,
                              err: S) -> Result<(), String> {
        match self.accept(token_types) {
            true => { self.scanner.ignore(); Ok(()) },
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

    //fn synchronize(&mut self) {
        //// sync on statement boundaries (ie: semicolon)
        //// TODO: check for loops' semicolon
        //while let Some(token) = self.scanner.next() {
            //if token.token == TT::SEMICOLON {
                //return self.scanner.ignore();
            //}
        //}
    //}
}


/* Grammar:
 *
 *  expression     := equality ;
 *  equality       := comparison { ( "!=" | "==" ) comparison } ;
 *  comparison     := addition { ( ">" | ">=" | "<" | "<=" ) addition } ;
 *  addition       := multiplication { ( "-" | "+" ) multiplication } ;
 *  multiplication := unary { ( "/" | "*" ) unary } ;
 *  unary          := ( "!" | "-" ) unary
 *                  | primary ;
 *  primary        := NUMBER | STRING | "false" | "true" | "nil"
 *                  | "(" expression ")" ;
 *
 */

impl LoxParser {
    fn expression(&mut self) -> ExprResult {
        self.equality()
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

    fn unary(&mut self) -> ExprResult {
        if self.accept(vec![TT::BANG, TT::MINUS]) {
            let op = self.scanner.extract().swap_remove(0);
            let rhs = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(rhs)));
        }
        self.primary()
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
        if self.accept(vec![TT::OPAREN]) {
            self.scanner.ignore(); // skip OPAREN
            let expr = self.expression()?;
            self.consume(vec![TT::CPAREN], "expect ')' after expression")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        let bad_token = self.scanner.peek();
        Err(self.error(bad_token, "expected expression"))
    }

    pub fn parse(&mut self) -> ExprResult {
        self.expression()
    }
}
