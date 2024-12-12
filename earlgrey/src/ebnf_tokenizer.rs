pub struct EbnfTokenizer<I: Iterator<Item = char>> {
    input: std::iter::Peekable<I>,
    buff: Vec<String>,
}

impl<I: Iterator<Item = char>> EbnfTokenizer<I> {
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
            Some(x) if "[]{}()|;".contains(x) => Ok(Some(x.to_string())),
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
            // Tags (starts with '@') and identifiers.
            Some(x) if x.is_ascii_alphabetic() || x == '@' || x == '_' => {
                let mut id = x.to_string();
                while let Some(ch) = self.input.peek() {
                    if !ch.is_ascii_alphanumeric() && *ch != '_' {
                        break;
                    }
                    id.push(self.input.next().unwrap());
                }
                Ok(Some(id))
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

impl<I: Iterator<Item = char>> Iterator for EbnfTokenizer<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_result() {
            Err(e) => {
                eprintln!("EbnfTokenizer error: {}", e);
                None
            }
            Ok(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EbnfTokenizer;

    #[test]
    fn simple() {
        let input = r#"
            "hello" world @tag | [ foo ];
            {x} # comment
            a:=(y)
            "escapedstring\""
        "#;
        let expected = vec![
            "\"",
            "hello",
            "\"",
            "world",
            "@tag",
            "|",
            "[",
            "foo",
            "]",
            ";",
            "{",
            "x",
            "}",
            "a",
            ":=",
            "(",
            "y",
            ")",
            "\"",
            "escapedstring\\\"",
            "\"",
        ];
        for (idx, token) in EbnfTokenizer::new(input.chars()).enumerate() {
            assert_eq!(token, expected[idx]);
        }
    }
}
