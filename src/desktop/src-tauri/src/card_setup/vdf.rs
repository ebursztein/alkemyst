/// Extract quoted/unquoted string pairs from bounded Steam configuration files.
/// Braces delimit pairs; comments and escaped quotes never become path keys.
pub(super) fn values(source: &str, key: &str) -> Vec<String> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() || c == '\u{feff}' => {}
            '{' | '}' => tokens.push(None),
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            }
            '"' => {
                let mut value = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    if c == '"' {
                        closed = true;
                        break;
                    }
                    if c == '\\' && matches!(chars.peek(), Some('\\' | '"')) {
                        value.push(chars.next().expect("peeked character"));
                    } else {
                        value.push(c);
                    }
                    if value.len() > 32_768 {
                        return vec![];
                    }
                }
                if !closed {
                    break;
                }
                tokens.push(Some(value));
            }
            c => {
                let mut value = String::from(c);
                while chars
                    .peek()
                    .is_some_and(|c| !c.is_whitespace() && !matches!(c, '{' | '}'))
                {
                    value.push(chars.next().expect("peeked character"));
                    if value.len() > 32_768 {
                        return vec![];
                    }
                }
                tokens.push(Some(value));
            }
        }
    }
    let mut pending: Option<String> = None;
    let mut values = vec![];
    for token in tokens {
        match token {
            None => pending = None,
            Some(value) => {
                if let Some(name) = pending.take() {
                    if name.eq_ignore_ascii_case(key) && !value.contains('\0') {
                        values.push(value);
                    }
                } else {
                    pending = Some(value);
                }
            }
        }
    }
    values
}

#[cfg(test)]
#[path = "../../tests/unit/card_vdf.rs"]
mod tests;
