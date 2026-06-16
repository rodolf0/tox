pub struct Tokenizer<'a> {
    inner: lexers::StringTokenizer<'a, std::str::Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let inner = lexers::StringTokenizer::new(input.chars())
            // Discard comments from '#' to end of line
            .split_by(
                |s| {
                    let cp = s.checkpoint();
                    if s.accept('#').is_some() {
                        s.accept_while(|c: &char| *c != '\n');
                        return Some(s.view_from(cp));
                    }
                    None
                },
                true,
            )
            // Double-quoted strings
            .split_by(|s| lexers::quoted(s, "\"", "\"", Some('\\')), false)
            // Single-quoted strings
            .split_by(|s| lexers::quoted(s, "'", "'", Some('\\')), false)
            // Numbers
            .split_by(lexers::number, false)
            // Identifiers / symbols
            .split_by(lexers::identifier, false)
            // Operators and punctuation
            .split_by(
                |s| {
                    let cp = s.checkpoint();
                    static MULTI: &[&str] = &["/.", "->", ":="];
                    for op in MULTI {
                        if s.accept_seq(op.chars()).is_some() {
                            return Some(s.view_from(cp));
                        }
                        s.restore(cp);
                    }
                    static SINGLE: &[char] = &[
                        '[', ']', '{', '}', '(', ')', ',', '+', '-', '*', '/', '^', '!', '~', ';',
                        '%', '=',
                    ];
                    if s.accept(SINGLE).is_some() {
                        return Some(s.view_from(cp));
                    }
                    None
                },
                false,
            );
        Self { inner }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::Tokenizer;

    #[test]
    fn parse_numbers() {
        let input = "1 123e2 123e-3 123e+4 0.23 0.23e2 0.23e-3 0.23e+4";
        let expected = vec![
            "1", "123e2", "123e-3", "123e+4", "0.23", "0.23e2", "0.23e-3", "0.23e+4",
        ];
        for (idx, token) in Tokenizer::new(input).enumerate() {
            assert_eq!(token, expected[idx]);
        }
    }

    #[test]
    fn parse_combinations() {
        let surrounds = vec![("[", "]"), ("{", "}"), ("(", ")"), ("", "")];
        let infix_ops = vec![",", ":=", "+", "-", "*", "/", "^", "/.", "->", " "];
        let postfix_ops = vec!["!", ""];
        let prefix_ops = vec!["-", "!", ""];
        let tokens = vec!["1", "0.23", "0.23e+4", "'str1'", "Symbol2", ""];
        let heads = vec!["", "Sum"];

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
                                                    expect.push(&lhs);
                                                    expect.extend([post1, op, pfx3]);
                                                    expect.push(&rhs);
                                                    expect.extend([post2, close, post3]);
                                                    let expect: Vec<_> = expect
                                                        .into_iter()
                                                        .filter(|s| !s.trim().is_empty())
                                                        .map(|s| s.to_string())
                                                        .collect();

                                                    let tokenized: Vec<_> =
                                                        Tokenizer::new(&expr).collect();
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
        println!("Combos: {}", combos);
    }
}
