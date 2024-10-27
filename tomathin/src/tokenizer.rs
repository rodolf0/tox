pub struct Tokenizer<I: Iterator<Item = char>> {
    input: std::iter::Peekable<I>,
    buff: Vec<String>,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    pub fn new(input: I) -> Self {
        Self {
            input: input.peekable(),
            buff: Vec::new(),
        }
    }

    fn next_result(&mut self) -> Result<Option<String>, String> {
        if self.buff.len() > 0 {
            return Ok(Some(self.buff.remove(0)));
        }
        match self.input.next() {
            // Various single char tokens.
            Some(x) if "[]{}(),+-*/^!".contains(x) => Ok(Some(x.to_string())),
            // Assignment operator.
            Some(':') => match self.input.next() {
                Some('=') => Ok(Some(":=".to_string())),
                _ => Err("Incomplete := operator".to_string()),
            },
            // Tokenize Strings checking for escapes.
            Some(open) if open == '"' || open == '\'' => {
                self.buff.push(open.to_string());
                let mut quoted_string = String::new();
                let mut escaped = false;
                while let Some(ch) = self.input.next() {
                    // Swallow escape char '\' and prevent string closure
                    if !escaped && ch == '\\' {
                        escaped = true;
                        continue;
                    }
                    if escaped {
                        quoted_string.push('\\')
                    }
                    // Found close token. Check it wasn't escaped.
                    if !escaped && open == ch {
                        self.buff.push(quoted_string);
                        self.buff.push(ch.to_string());
                        return self.next_result();
                    }
                    quoted_string.push(ch);
                    escaped = false;
                }
                Err("Unfinished string missing close quote".to_string())
            }
            // Swallow comments until EOL.
            Some('#') => {
                while let Some(nl) = self.input.next() {
                    if nl == '\n' {
                        return self.next_result();
                    }
                }
                Err("Unfinished comment missing EOL".to_string())
            }
            // Tokenize variable names
            Some(x) if x.is_ascii_alphabetic() || x == '_' => {
                let mut id = x.to_string();
                while let Some(ch) = self.input.peek() {
                    if !ch.is_ascii_alphanumeric() && *ch != '_' {
                        break;
                    }
                    id.push(self.input.next().unwrap());
                }
                Ok(Some(id))
            }
            // Tokenize numbers TODO: unary minus shouldn't be part of this
            Some(x) if x.is_ascii_digit() || x == '-' => {
                let mut num = x.to_string();
                // consume all integer part
                while let Some(n) = self.input.peek() {
                    if !n.is_ascii_digit() {
                        break;
                    }
                    num.push(self.input.next().unwrap());
                }
                // Maybe fractional part
                if let Some(dot) = self.input.peek() {
                    if *dot == '.' {
                        num.push(self.input.next().unwrap());
                        // consume fractional part
                        while let Some(n) = self.input.peek() {
                            if !n.is_ascii_digit() {
                                break;
                            }
                            num.push(self.input.next().unwrap());
                        }
                    }
                }
                // Maybe exponent
                if let Some(exp) = self.input.peek() {
                    if *exp == 'e' || *exp == 'E' {
                        num.push(self.input.next().unwrap());
                        // Optional exponent sign
                        if let Some(sign) = self.input.peek() {
                            if *sign == '-' || *sign == '+' {
                                num.push(self.input.next().unwrap());
                            }
                        }
                        // consume exponent digits
                        while let Some(n) = self.input.peek() {
                            if !n.is_ascii_digit() {
                                break;
                            }
                            num.push(self.input.next().unwrap());
                        }
                    }
                }
                Ok(Some(num))
            }
            // Swallow whitespace.
            Some(x) if x.is_whitespace() => {
                while let Some(ws) = self.input.peek() {
                    if !ws.is_whitespace() {
                        break;
                    }
                    self.input.next(); // consume whitespace
                }
                self.next_result()
            }
            Some(ch) => Err(format!("Unexpected char: {}", ch)),
            None => Ok(None),
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_result() {
            Err(e) => {
                eprintln!("Tokenizer error: {}", e);
                None
            }
            Ok(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tokenizer;

    #[test]
    fn parse_numbers() {
        // Full number 123.456e-78
        let input = "1 123e2 123e-3 123e+4 0.23 0.23e2 0.23e-3 0.23e+4";
        let expected = vec![
            "1", "123e2", "123e-3", "123e+4", "0.23", "0.23e2", "0.23e-3", "0.23e+4",
        ];
        for (idx, token) in Tokenizer::new(input.chars()).enumerate() {
            assert_eq!(token, expected[idx]);
        }
    }

    #[test]
    fn parse_combinations() {
        let surrounds = vec![("[", "]"), ("{", "}"), ("(", ")"), ("", "")];
        let infix_ops = vec![",", ":=", "+", "-", "*", "/", "^", " "];
        let postfix_ops = vec!["!", ""];
        let prefix_ops = vec!["-", "!", ""];
        let tokens = vec!["1", "0.23", "0.23e+4", "'str1'", "Symbol2", ""];
        let heads = vec!["", "Sum"];

        // "# comment\n",
        let mut combos = 0;
        for head in &heads {
            for (open, close) in &surrounds {
                for pfx1 in &prefix_ops {
                    for pfx2 in &prefix_ops {
                        for pfx3 in &prefix_ops {
                            for post1 in &postfix_ops {
                                for post2 in &postfix_ops {
                                    for post3 in &postfix_ops {
                                        for op in &infix_ops {
                                            for token_pairs in tokens.windows(2) {
                                                if let &[lhs, rhs] = token_pairs {
                                                    if *open == "" && *pfx2 == "" {
                                                        continue;
                                                    }
                                                    let expr = format!(
                                                        "{}{}{}{}{}{}{}{}{}{}{}{}",
                                                        pfx1,
                                                        head,
                                                        open,
                                                        pfx2,
                                                        lhs,
                                                        post1,
                                                        op,
                                                        pfx3,
                                                        rhs,
                                                        post2,
                                                        close,
                                                        post3
                                                    );
                                                    let mut expect = vec![pfx1, head, open, pfx2];
                                                    if lhs == "'str1'" {
                                                        expect.extend([&"'", &"str1", &"'"]);
                                                    } else {
                                                        expect.push(&lhs);
                                                    }
                                                    expect.extend([post1, op, pfx3]);
                                                    if rhs == "'str1'" {
                                                        expect.extend([&"'", &"str1", &"'"]);
                                                    } else {
                                                        expect.push(&rhs);
                                                    }
                                                    expect.extend([post2, close, post3]);
                                                    let expect: Vec<_> = expect
                                                        .into_iter()
                                                        .filter(|s| !s.trim().is_empty())
                                                        .map(|s| s.to_string())
                                                        .collect();

                                                    let tokenized: Vec<_> =
                                                        Tokenizer::new(expr.chars()).collect();
                                                    assert_eq!(tokenized, expect);
                                                    combos += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        eprintln!("Combos: {}", combos);
    }
}
