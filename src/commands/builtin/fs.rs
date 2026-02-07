use std::{env, path::PathBuf};

use crate::commands::error::CommandsError;

pub(crate) fn invoke_pwd(_cmd_list: &[String]) -> Result<String, CommandsError> {
    let curr = match env::current_dir() {
        Ok(path) => path,
        Err(e) => return Err(CommandsError::InvalidCurrentDirectory(e))?,
    };
    Ok(format!("{}\n", curr.display()))
}

pub(crate) fn invoke_cd(cmd_list: &[String]) -> Result<Option<String>, CommandsError> {
    let cmd_list: Vec<_> = cmd_list.iter().collect();
    if cmd_list.is_empty() {
        return Ok(None);
    }

    let path = match cmd_list[0].as_ref() {
        "~" => match std::env::var_os("HOME") {
            Some(home_path) => PathBuf::from(home_path),
            None => return Err(CommandsError::HomeNotSet)?,
        },
        _ => PathBuf::from(&cmd_list[0]),
    };
    if path.exists() {
        // if cd fails then proceed to next REPL iter
        let _ = env::set_current_dir(path);
        Ok(None)
    } else {
        Ok(Some(format!(
            "cd: {}: No such file or directory\n",
            path.display()
        )))
    }
}
