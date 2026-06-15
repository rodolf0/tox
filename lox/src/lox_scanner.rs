use lexers::TypedTokenizer;
use lexers::{identifier, number};
use std::cell::Cell;
use std::rc::Rc;

#[rustfmt::skip]
#[derive(Clone,Debug,PartialEq)]
pub enum TT {
    // single char tokens
    OPAREN, CPAREN, OBRACE, CBRACE, COMMA, DOT,
    MINUS, PLUS, SEMICOLON, SLASH, STAR, DOLLAR,
    BANG, ASSIGN, NE, EQ, GT, GE, LT, LE,
    // literals
    Id(String), Str(String), Num(f64),
    // keywords
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR, BREAK,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub line: usize,
    pub token: TT,
    pub lexeme: String,
}

pub fn scanner<'a, I: Iterator<Item = char> + 'a>(source: I) -> impl Iterator<Item = Token> + 'a {
    let line = Rc::new(Cell::new(1));

    let id_or_keyword = move |lexeme: String, line: usize| -> Token {
        let token = match lexeme.as_str() {
            "and" => TT::AND,
            "class" => TT::CLASS,
            "else" => TT::ELSE,
            "false" => TT::FALSE,
            "fun" => TT::FUN,
            "for" => TT::FOR,
            "if" => TT::IF,
            "nil" => TT::NIL,
            "or" => TT::OR,
            "break" => TT::BREAK,
            "print" => TT::PRINT,
            "return" => TT::RETURN,
            "super" => TT::SUPER,
            "this" => TT::THIS,
            "true" => TT::TRUE,
            "var" => TT::VAR,
            "while" => TT::WHILE,
            _ => TT::Id(lexeme.clone()),
        };
        Token {
            line,
            token,
            lexeme,
        }
    };

    TypedTokenizer::new(source, {
        let line = line.clone();
        move |chars| {
            let lexeme: String = chars.iter().collect();
            if lexeme == "\"" {
                eprintln!("LoxScanner error: unterminated string");
            } else {
                eprintln!(
                    "LoxScanner error: bad char '{}' at line {}",
                    lexeme,
                    line.get()
                );
            }
            None
        }
    })
    .trimmer(|c| *c == ' ' || *c == '\t' || *c == '\r')
    .split_on("\n", {
        let line = line.clone();
        move |_| {
            line.set(line.get() + 1);
            None
        }
    })
    .split_by(
        |s| {
            let cp = s.checkpoint();
            if s.accept_seq("//".chars()).is_some() {
                s.accept_while(|c: &char| *c != '\n');
                Some(s.view_from(cp))
            } else {
                None
            }
        },
        move |_| None,
    )
    .split_by(|s| lexers::quoted(s, "\"", "\"", Some('\\')), {
        let line = line.clone();
        move |chars| {
            let s: String = chars.iter().collect();
            let newlines = s.chars().filter(|&c| c == '\n').count();
            let current_line = line.get();
            line.set(current_line + newlines);
            let inner: String = chars[1..chars.len() - 1].iter().collect();
            Some(Token {
                line: current_line,
                token: TT::Str(inner),
                lexeme: s,
            })
        }
    })
    .split_on(identifier, {
        let line = line.clone();
        move |chars| {
            let lexeme: String = chars.iter().collect();
            Some(id_or_keyword(lexeme, line.get()))
        }
    })
    .split_on(number, {
        let line = line.clone();
        move |chars| {
            let lexeme: String = chars.iter().collect();
            use std::str::FromStr;
            let val = f64::from_str(&lexeme).unwrap();
            Some(Token {
                line: line.get(),
                token: TT::Num(val),
                lexeme,
            })
        }
    })
    .split_on(
        [
            "!=", "==", "<=", ">=", "(", ")", "{", "}", ",", ".", "-", "+", ";", "*", "$", "!",
            "=", "<", ">", "/",
        ],
        {
            let line = line.clone();
            move |chars| {
                let lexeme: String = chars.iter().collect();
                let tok = match lexeme.as_str() {
                    "(" => TT::OPAREN,
                    ")" => TT::CPAREN,
                    "{" => TT::OBRACE,
                    "}" => TT::CBRACE,
                    "," => TT::COMMA,
                    "." => TT::DOT,
                    "-" => TT::MINUS,
                    "+" => TT::PLUS,
                    ";" => TT::SEMICOLON,
                    "*" => TT::STAR,
                    "$" => TT::DOLLAR,
                    "!" => TT::BANG,
                    "=" => TT::ASSIGN,
                    "!=" => TT::NE,
                    "==" => TT::EQ,
                    ">" => TT::GT,
                    ">=" => TT::GE,
                    "<" => TT::LT,
                    "<=" => TT::LE,
                    "/" => TT::SLASH,
                    _ => return None,
                };
                Some(Token {
                    line: line.get(),
                    token: tok,
                    lexeme,
                })
            }
        },
    )
}
