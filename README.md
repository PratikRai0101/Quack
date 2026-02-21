Quack
=====

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/PratikRai0101/Quack/actions) [![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Quack is a small, fast Rust CLI that replays a failing shell command, captures stdout/stderr, and asks a streaming Groq LLM (or a compatible streaming LLM) for a concise root-cause analysis and an immediate fix. Answers stream into a terminal TUI so you can act immediately.

Quack focuses on one thing: provide a corrected command you can run right away — focused, actionable fixes.

Table of contents
- Background
- Features
- Install
- Usage
- Shell integration (one-click)
- Development
- Troubleshooting
- Contributing
- License

Background
----------

Developers spend too much time switching between the terminal, search results, and docs when a command errors. Quack automates the triage: replay the command, capture output, and ask a high-quality model for a precise fix — streamed into your terminal so you can act immediately.

Features
--------

- Replays commands through your shell so quoting and flags are preserved (uses `SHELL -c "..."`).
- Reads the last command from Bash/Zsh/Fish history with robust parsing and filters to avoid self-invocation.
- Native TUI using `ratatui` + `crossterm`: top pane shows combined stdout/stderr, bottom pane streams the AI answer (wraps long lines).
- Streaming LLM integration (Groq) with an expert system prompt tailored to your OS and package manager.
- `quack init` installs a safe shell wrapper that flushes history before running — avoids stale history.
- Graceful shutdown, non-blocking event loop, and zero-warning build hygiene.

Install
-------

Prerequisites

- Rust toolchain (stable)
- A Groq API key in `GROQ_API_KEY` for streaming model responses (optional for local testing)

Build from source

```bash
git clone git@github.com:PratikRai0101/Quack.git
cd Quack
cargo build --release
# optional: cargo install --path .
```

One-click install & setup
-------------------------

For a quick install and automatic shell integration, run this one-liner (requires Rust toolchain):

```bash
git clone git@github.com:PratikRai0101/Quack.git && cd Quack && cargo install --path . && quack init
```

What this does:

- Clones the repository and builds/installs the `quack` binary into your cargo bin directory.
- Runs `quack init`, which appends the safe `quack` wrapper into your shell RC (detects your shell and updates the appropriate file). This wrapper flushes history before running Quack so the latest command is replayed.

After running the one-liner either restart your shell or source the updated rc file, e.g.:

```bash
# restart or source your shell rc file
```

If you prefer not to install globally, build and run locally:

```bash
cargo run -- --cmd "echo hello"
```

Usage details
-------------

Quick Start

1. Ensure `GROQ_API_KEY` is set (or run without model integration for local debugging).
2. Run `quack --cmd "<failing command>"` or `quack` to replay the last history entry.
3. TUI: Top pane shows the command output; bottom pane streams a structured, scannable expert response with the corrected command.
4. Quit with `q` or `Esc`.

Key options

- `--cmd <STR>` : replay this command instead of reading history
- `quack init`  : append the wrapper to your shell rc (fish/zsh/bash)

Quick example
-------------

Run Quack against a failing command to see a streamed suggestion:

```bash
quack --cmd "ls /nonexistent"
```

Shell integration (one-click)
---------------------------

Quack includes `quack init` which appends a small wrapper to your shell rc file to flush history before running the binary.

Example installed wrappers:

- Fish (`~/.config/fish/config.fish`)

```fish
function quack
    history save
    command quack $argv
end
```

- Zsh (`~/.zshrc`)

```bash
quack() {
    fc -W
    command quack "$@"
}
```

- Bash (`~/.bashrc`)

```bash
quack() {
    history -a
    command quack "$@"
}
```

Use:

```bash
# append wrapper for your detected shell
quack init

# or print and source manually
quack init
# then source the file or restart your shell
```

The init command uses `dirs::home_dir()` to find the correct rc file and avoids adding duplicates.

Usage details
-------------

Quick Start

1. Ensure `GROQ_API_KEY` is set (or run without model integration for local debugging).
2. Run `quack --cmd "<failing command>"` or `quack` to replay the last history entry.
3. TUI: Top pane shows the command output; bottom pane streams a structured, scannable expert response with the corrected command.
4. Quit with `q` or `Esc`.

Development
-----------

- Run tests:

```bash
cargo test
```

- Run in dev mode:

```bash
cargo run -- --cmd "echo hello"
```

Files of interest

- `src/main.rs` — CLI, TUI event loop, OS detection, `quack init` implementation
- `src/shell.rs` — history parsing, `get_last_command()`, `replay_command()` (uses user shell)
- `src/groq.rs` — Groq streaming client + system prompt (Scannable Expert format)
- `src/tui.rs` — UI rendering using `ratatui`
- `src/context.rs` — optional git diff for additional context

Troubleshooting
---------------

- If Quack shows empty panes, ensure your shell writes history to disk. Use `history -a` (bash) or `fc -W` (zsh), or run `quack init` which installs a wrapper that flushes history.
- Fish history is parsed strictly to avoid reading `when:` timestamp lines incorrectly. If your fish version stores commands differently, you might need to adjust `src/shell.rs`.
- If the AI response is not structured, ensure `GROQ_API_KEY` is valid and network connectivity is available.

Contributing
------------

Contributions welcome. Suggested workflow:

1. Fork and create a feature branch
2. Add tests for parsing or behavior where appropriate
3. Open a PR describing changes and rationale

Please run `cargo test` and `cargo clippy` before submitting PRs.

License
-------

MIT (or change to your preferred license). See `LICENSE`.

Acknowledgements
----------------

Quack uses:

- tokio, reqwest for async HTTP + streaming
- ratatui + crossterm for terminal UI
- serde / serde_json for payload handling

Support or questions
--------------------

Open an issue or discussion on the repository; include the failing command and any relevant logs.
