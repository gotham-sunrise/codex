# Anecdoct CLI (Rust Implementation)

We provide Anecdoct CLI as a standalone executable to ensure a zero-dependency install.

## Installing Anecdoct

Today, the easiest way to install Anecdoct is via `npm`:

```shell
npm i -g @openai/codex
anecdoct
```

You can also install via Homebrew (`brew install --cask anecdoct`) or download a platform-specific release directly from our [GitHub Releases](https://github.com/lambogenius/anecdoct/releases).

## Documentation quickstart

- First run with Anecdoct? Start with [`docs/getting-started.md`](../docs/getting-started.md) (links to the walkthrough for prompts, keyboard shortcuts, and session management).
- Want deeper control? See [`docs/config.md`](../docs/config.md) and [`docs/install.md`](../docs/install.md).

## What's new in the Rust CLI

The Rust implementation is now the maintained Anecdoct CLI and serves as the default experience. It includes a number of features that the legacy TypeScript CLI never supported.

### Config

Anecdoct supports a rich set of configuration options. Note that the Rust CLI uses `config.toml` instead of `config.json`. See [`docs/config.md`](../docs/config.md) for details.

### Model Context Protocol Support

#### MCP client

Anecdoct CLI functions as an MCP client that allows the Anecdoct CLI and IDE extension to connect to MCP servers on startup. See the [`configuration documentation`](../docs/config.md#connecting-to-mcp-servers) for details.

#### MCP server (experimental)

Anecdoct can be launched as an MCP _server_ by running `anecdoct mcp-server`. This allows _other_ MCP clients to use Anecdoct as a tool for another agent.

Use the [`@modelcontextprotocol/inspector`](https://github.com/modelcontextprotocol/inspector) to try it out:

```shell
npx @modelcontextprotocol/inspector anecdoct mcp-server
```

Use `anecdoct mcp` to add/list/get/remove MCP server launchers defined in `config.toml`, and `anecdoct mcp-server` to run the MCP server directly.

### Notifications

You can enable notifications by configuring a script that is run whenever the agent finishes a turn. The [notify documentation](../docs/config.md#notify) includes a detailed example that explains how to get desktop notifications via [terminal-notifier](https://github.com/julienXX/terminal-notifier) on macOS. When Anecdoct detects that it is running under WSL 2 inside Windows Terminal (`WT_SESSION` is set), the TUI automatically falls back to native Windows toast notifications so approval prompts and completed turns surface even though Windows Terminal does not implement OSC 9.

### `anecdoct exec` to run Anecdoct programmatically/non-interactively

To run Anecdoct non-interactively, run `anecdoct exec PROMPT` (you can also pass the prompt via `stdin`) and Anecdoct will work on your task until it decides that it is done and exits. If you provide both a prompt argument and piped stdin, Anecdoct appends stdin as a `<stdin>` block after the prompt so patterns like `echo "my output" | anecdoct exec "Summarize this concisely"` work naturally. Output is printed to the terminal directly. You can set the `RUST_LOG` environment variable to see more about what's going on.
Use `anecdoct exec --ephemeral ...` to run without persisting session rollout files to disk.

### Experimenting with the Anecdoct Sandbox

To test to see what happens when a command is run under the sandbox provided by Anecdoct, we provide the following subcommands in Anecdoct CLI:

```
# macOS
anecdoct sandbox macos [--log-denials] [COMMAND]...

# Linux
anecdoct sandbox linux [COMMAND]...

# Windows
anecdoct sandbox windows [COMMAND]...

# Legacy aliases
anecdoct debug seatbelt [--log-denials] [COMMAND]...
anecdoct debug landlock [COMMAND]...
```

To try a writable legacy sandbox mode with these commands, pass an explicit config override such
as `-c 'sandbox_mode="workspace-write"'`.

### Selecting a sandbox policy via `--sandbox`

The Rust CLI exposes a dedicated `--sandbox` (`-s`) flag that lets you pick the sandbox policy **without** having to reach for the generic `-c/--config` option:

```shell
# Run Anecdoct with the default, read-only sandbox
anecdoct --sandbox read-only

# Allow the agent to write within the current workspace while still blocking network access
anecdoct --sandbox workspace-write

# Danger! Disable sandboxing entirely (only do this if you are already running in a container or other isolated env)
anecdoct --sandbox danger-full-access
```

The same setting can be persisted in `~/.anecdoct/config.toml` via the top-level `sandbox_mode = "MODE"` key, e.g. `sandbox_mode = "workspace-write"`.
In `workspace-write`, Anecdoct also includes `~/.anecdoct/memories` in its writable roots so memory maintenance does not require an extra approval.

## Code Organization

This folder is the root of a Cargo workspace. It contains quite a bit of experimental code, but here are the key crates:

- [`core/`](./core) contains the business logic for Anecdoct. Ultimately, we hope this becomes a library crate that is generally useful for building other Rust/native applications that use Anecdoct.
- [`exec/`](./exec) "headless" CLI for use in automation.
- [`tui/`](./tui) CLI that launches a fullscreen TUI built with [Ratatui](https://ratatui.rs/).
- [`cli/`](./cli) CLI multitool that provides the aforementioned CLIs via subcommands.

If you want to contribute or inspect behavior in detail, start by reading the module-level `README.md` files under each crate and run the project workspace from the top-level `anecdoct-rs` directory so shared config, features, and build scripts stay aligned.
