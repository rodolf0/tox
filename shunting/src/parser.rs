use lexers::{MathToken, MathTokenizer};

#[derive(PartialEq, Debug)]
pub enum Assoc {
    Left,
    Right,
}

pub fn op_precedence(mt: &MathToken) -> Result<(usize, Assoc), String> {
    // NOTE: This can't encode relations between all tokens, just Ops.
    // For example:
    // In https://github.com/rodolf0/natools/blob/master/libparser/parser.c#L56-L94
    // - unary-minus has to be < than Numbers and OParen
    // - but OParen has to be < than unary-minus too!
    // - At the same time, unary-minus has to be > than bin-ops (eg: +)
    Ok(match mt {
        MathToken::BOp(o) if o == "+" => (2, Assoc::Left),
        MathToken::BOp(o) if o == "-" => (2, Assoc::Left),
        MathToken::BOp(o) if o == "*" => (3, Assoc::Left),
        MathToken::BOp(o) if o == "/" => (3, Assoc::Left),
        MathToken::BOp(o) if o == "%" => (3, Assoc::Left),
        MathToken::BOp(o) if o == "^" || o == "**" => (4, Assoc::Right),
        MathToken::UOp(o) if o == "-" => (5, Assoc::Right), // unary minus
        MathToken::UOp(o) if o == "!" => (6, Assoc::Left), // factorial
        _ => return Err(format!("Undefined precedence for {:?}", mt)),
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct RPNExpr(pub Vec<MathToken>);

pub struct ShuntingParser;

impl ShuntingParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, String> {
        Self::parse(&mut MathTokenizer::new(expr.chars()))
    }

    pub fn parse(lexer: &mut impl Iterator<Item = MathToken>) -> Result<RPNExpr, String> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        for token in lexer {
            match token {
                MathToken::Number(_) => out.push(token),
                MathToken::Variable(_) => out.push(token),
                MathToken::OParen => stack.push(token),
                MathToken::Function(_, _) => {
                    stack.push(token);
                    arity.push(1);
                }
                MathToken::Comma | MathToken::CParen => {
                    // Flush stack to output queue until open paren
                    loop {
                        match stack.pop() {
                            // Only advance until we find the matching open paren
                            Some(MathToken::OParen) => break,
                            Some(any) => out.push(any),
                            None => return Err("Missing Opening Paren".to_string()),
                        }
                    }
                    if token == MathToken::Comma {
                        // Keep track of function arity based on number of commas
                        stack.push(MathToken::OParen); // put back OParen if reading Comma
                        match arity.last_mut() {
                            Some(a) => *a += 1,
                            None => return Err("Comma outside function arglist".to_string())
                        }
                    } else if let Some(MathToken::Function(fname, _)) = stack.last() {
                        // token is CParen. Popped everything up to OParen. Check fn call.
                        out.push(MathToken::Function(fname.clone(), arity.pop().unwrap()));
                        stack.pop(); // pop the function we just shifted out
                    }
                }
                MathToken::UOp(_) | MathToken::BOp(_) => {
                    let (input_token_prec, input_token_assoc) = op_precedence(&token)?;
                    // Flush stack while its precedence is lower than input or reach OParen
                    while let Some(stack_top) = stack.last() {
                        if stack_top == &MathToken::OParen {
                            break;
                        }
                        let (stack_top_prec, _) = op_precedence(stack_top)?;
                        if stack_top_prec < input_token_prec || (
                            stack_top_prec == input_token_prec &&
                            input_token_assoc == Assoc::Right) {
                            break;
                        }
                        out.push(stack.pop().unwrap());
                    }
                    stack.push(token);
                }
                MathToken::Quantity(_, _, _) => return Err("Can't handle quantities".to_string()),
                MathToken::Unknown(lexeme) => return Err(format!("Bad token: {}", lexeme)),
            }
        }
        while let Some(top) = stack.pop() {
            match top {
                MathToken::OParen => return Err("Missing Closing Paren".to_string()),
                token => out.push(token),
            }
        }
        Ok(RPNExpr(out))
    }
}
