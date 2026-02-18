[![progress-banner](https://backend.codecrafters.io/progress/shell/58e14eb1-d9f8-4481-a18b-ed98a125946d)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a shell built for the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# 🦀 Clawsh
A small, sharp, Rust‑powered shell with bite.

Clawsh is a Unix‑style shell written in Rust.   
It supports pipelines, redirections, builtins, and external commands.  
Originally built as a Codecrafters challenge, it has grown and popped it's shell onto crates.io.

## Features

- Builtin commands (`cd`, `pwd`, `echo`, `type`, `history`)
- Pipelines (`ls | grep foo | wc -l`)
- Redirections (`>`, `>>`, `2>`, `2>>`)
- External command execution
- Persistent history with append/read/write modes

---

## Installation

### From crates.io

```
cargo install clawsh
```

---

## Usage

Start the shell:

```
clawsh
```

Examples:

```
clawsh
$> echo hello world
hello world

clawsh
$> ls | grep rs | wc -l

clawsh
$> history -a ~/.clawsh_history
```

---

## Roadmap

- [] add unit tests and integration tests based on codecrafters test suite
- [] add docs
- [] implement more builtin commands when codecrafters updates their course

---

## License

MIT or Apache-2.0
