#[must_use]
pub fn split_words(input: &str) -> Vec<String> {
    let mut command_list: Vec<String> = vec![];
    let mut buf = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                if in_single_quotes || in_double_quotes {
                    buf.push(c);
                } else {
                    if buf.is_empty() {
                        continue;
                    }
                    command_list.push(buf.clone());
                    buf.clear();
                }
            }
            '\\' => {
                if !in_single_quotes
                    && !in_double_quotes
                    && let Some(next_char) = chars.next()
                {
                    buf.push(next_char);
                }
                if in_single_quotes {
                    buf.push(c);
                }
                if in_double_quotes && let Some(&c) = chars.peek() {
                    // unwrap safe as the peek returns Some
                    match c {
                        '\"' | '\\' => buf.push(chars.next().unwrap()),
                        _ => buf.push('\\'),
                    }
                }
            }
            '\'' => {
                if in_double_quotes {
                    buf.push(c);
                    continue;
                }
                in_single_quotes = !in_single_quotes;
            }
            '\"' => {
                if in_single_quotes {
                    buf.push(c);
                } else {
                    in_double_quotes = !in_double_quotes;
                }
            }
            _ => buf.push(c),
        }
    }
    if !buf.is_empty() {
        command_list.push(buf.clone());
    }
    command_list
}
