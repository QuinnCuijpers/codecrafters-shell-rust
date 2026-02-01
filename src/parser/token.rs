#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Command(String),
    Redirect(String),
    Arg(String),
    Pipe,
}

#[must_use]
pub fn tokenize_input(input: Vec<String>) -> Option<Vec<Token>> {
    if input.is_empty() {
        eprintln!("input string was empty when attempted tokinization");
        return None;
    }
    let mut tokenized = vec![];
    let mut iter = input.into_iter();
    tokenized.push(Token::Command(iter.next().unwrap())); //first String always exists by the above if case

    let mut new_command = false;
    for s in iter {
        match s.as_str() {
            ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => tokenized.push(Token::Redirect(s)),
            "|" => {
                new_command = true;
                tokenized.push(Token::Pipe);
            }
            _ => {
                if new_command {
                    tokenized.push(Token::Command(s));
                    new_command = !new_command;
                } else {
                    tokenized.push(Token::Arg(s));
                }
            }
        }
    }

    Some(tokenized)
}
