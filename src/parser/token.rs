#[derive(Debug, Clone, PartialEq, Eq)]
/// Enum representing the different types of tokens that can be parsed from user input
pub enum Token {
    /// The command token, representing the command to be executed
    Command(String),
    /// The redirect token, representing a redirection operator (e.g. `>`, `>>`, `1>`, `2>`, etc.)
    Redirect(String),
    /// The argument token, representing an argument to a command
    Arg(String),
    /// The pipe token, representing the pipe operator (`|`) connecting two commands
    Pipe,
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
/// Tokenize the input vector of strings into a vector of `Token`s, returning `None` if the input is empty
pub fn tokenize_input(input: Vec<String>) -> Option<Vec<Token>> {
    // design wise we decided to have this function return an owned vec instead of an iterator
    // as the vec will always be small enough
    if input.is_empty() {
        return None;
    }
    let mut tokenized = vec![];
    let mut iter = input.into_iter();
    #[allow(clippy::expect_used)]
    tokenized.push(Token::Command(
        iter.next()
            .expect("first String always exists by the above if case"),
    ));

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
