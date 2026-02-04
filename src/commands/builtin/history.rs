use rustyline::history::{FileHistory, History, SearchDirection};
use std::{
    cmp::min,
    collections::HashSet,
    fs::{File, read, write},
    io::Write,
    path::Path,
};

pub(crate) fn invoke_history(args_str: &[String], history: &mut FileHistory) -> Option<String> {
    use std::fmt::Write;
    let mut args_iter = args_str.iter();
    let length = if let Some(arg) = args_iter.next() {
        match arg.parse::<usize>() {
            Ok(n) => min(n, history.len()),
            Err(_) => match arg.as_ref() {
                "-r" => {
                    invoke_history_read_file(history, &mut args_iter);
                    0
                }
                "-w" => {
                    invoke_history_write(history, &mut args_iter);
                    0
                }
                "-a" => {
                    invoke_history_append(history, args_iter);
                    0
                }
                _ => history.len(),
            },
        }
    } else {
        history.len()
    };
    let mut buf = String::new();
    for i in 0..length {
        let entry_idx = history.len() - length + i;
        #[allow(clippy::expect_used)]
        let search_result = history
            .get(entry_idx, SearchDirection::Reverse)
            .expect("Rustyline implementation of get on HistoryFile can not error");
        if let Some(entry) = search_result {
            let entry = entry.entry;
            let _ = writeln!(buf, "  {} {}", entry_idx + 1, entry);
        }
    }
    if buf.is_empty() { None } else { Some(buf) }
}

fn invoke_history_write(history: &mut FileHistory, args_iter: &mut std::slice::Iter<'_, String>) {
    if let Some(file_name) = args_iter.next() {
        let mut new_contents = vec![];
        for entry in history.iter() {
            let mut new_entry = entry.clone();
            new_entry.push('\n');
            new_contents.append(&mut new_entry.as_bytes().to_owned());
        }
        let _ = write(file_name, new_contents);
    }
}

fn invoke_history_append(history: &mut FileHistory, mut args_iter: std::slice::Iter<'_, String>) {
    if let Some(file_name) = args_iter.next() {
        let mut file_options = File::options();
        file_options.create(true).write(true).append(true);
        if let Ok(mut file) = file_options.open(file_name)
            && let Ok(old_contents) = read(file_name)
        {
            let mut set = HashSet::new();

            let old_string = String::from_utf8_lossy(&old_contents);
            for s in old_string.lines() {
                set.insert(s);
            }

            let mut last_append_index = None;

            for (i, entry) in history.iter().rev().skip(1).enumerate() {
                if entry.starts_with("history -a") {
                    last_append_index = Some(history.len() - 2 - i);
                    break;
                }
            }

            let start = last_append_index.map_or(0, |i| i + 1);

            let mut new_entries = Vec::new();

            for entry in history.iter().skip(start) {
                if !entry.starts_with("history -a") {
                    new_entries.push(entry.clone());
                }
            }

            let mut written = false;
            for entry in new_entries {
                if set.contains(entry.as_str()) {
                    continue;
                }
                _ = file.write_all(entry.as_bytes());
                _ = file.write_all(b"\n");
                written = true;
            }

            if written {
                _ = file.write_all(format!("history -a {file_name}\n").as_bytes());
            }
        }
    }
}

fn invoke_history_read_file(
    history: &mut FileHistory,
    args_iter: &mut std::slice::Iter<'_, String>,
) {
    let env_file = std::env::var_os("HISTFILE");

    let history_file = if let Some(file_name) = env_file {
        _ = File::create(&file_name);
        file_name
    } else {
        _ = File::create("/tmp/history.txt");
        "/tmp/history.txt".into()
    };

    if let Some(file_name) = args_iter.next() {
        if history.load(Path::new(file_name)).is_err() {
            eprintln!("Could not read history from file {file_name}");
        } else {
            let mut new_contents = format!("history -r {file_name}\n").as_bytes().to_owned();
            if let Ok(mut contents) = read(file_name) {
                new_contents.append(&mut contents);
                let _ = write(&history_file, new_contents);
                let _ = history.load(Path::new(&history_file));
            }
        }
    }
}
