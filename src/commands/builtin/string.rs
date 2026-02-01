use crate::commands::{Builtin, find_exec_file};
use std::str::FromStr;

pub(crate) fn invoke_echo<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut s = cmd_list
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    s.push('\n');
    s
}

pub(crate) fn invoke_type<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use std::fmt::Write;
    let mut buf = String::new();
    for (i, cmd) in cmd_list.into_iter().enumerate() {
        let cmd = cmd.as_ref();
        let cmd_str = if i != 0 {
            format!("\n{cmd}")
        } else {
            cmd.to_string()
        };
        let cmd_str = cmd_str.as_str();
        if Builtin::from_str(cmd).is_ok() {
            let _ = write!(buf, "{cmd_str} is a shell builtin");
        }
        // go through every directory and check if a file with the name exist that has exec permissions
        else if let Some(file_path) = find_exec_file(cmd) {
            let _ = write!(buf, "{cmd_str} is {}", file_path.display());
        } else {
            let _ = write!(buf, "{cmd_str}: not found");
        }
    }
    buf.push('\n');
    buf
}
