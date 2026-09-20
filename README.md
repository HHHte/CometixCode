# CometixCode

A terminal-based AI coding assistant, written in Rust as a 1:1 reimplementation
of Anthropic's Claude Code.

> **Unofficial project, not affiliated with Anthropic.**
> This is an independent reimplementation built by reading the published
> Claude Code CLI. "Claude" and "Claude Code" are trademarks of Anthropic, PBC.
> Nothing here is endorsed by or supported by Anthropic.

## What this is

Claude Code's terminal UI is TypeScript on React + Ink. CometixCode reproduces
it in Rust on iocraft, a React-style retained-mode TUI framework that is the
structural counterpart of Ink — components, hooks, declarative elements. The
port follows the original file by file rather than reinventing the
architecture: a TypeScript component maps to a Rust component, a hook to a
hook, and deliberate deviations are recorded in the source where they happen.

It builds against [CometixTUI], a fork of iocraft carrying the primitives this
port needs (row-level diffing, event propagation, SIGCONT self-healing, IME
cursor, bracketed paste, grid layout).

It is a work in progress. Large parts of the interactive loop, tool execution,
permissions, MCP and slash commands are implemented; other areas are partial.

## Building

### Prerequisites

- **Rust 1.87+** (edition 2024)
- **ICU4C** — the `rust_icu_*` crates bind to it through `pkg-config`

On macOS, ICU4C is keg-only, so its `pkgconfig` directory has to be on the
search path:

```sh
brew install icu4c
export PKG_CONFIG_PATH="$(brew --prefix icu4c)/lib/pkgconfig"
```

On Debian/Ubuntu:

```sh
sudo apt install libicu-dev pkg-config
```

### Build and run

```sh
cargo build --release
cargo run --release
```

The binary is `cometix`.

## ripgrep

File search shells out to `rg`. Claude Code ships a packaged ripgrep inside its
npm package and executes that path; CometixCode keeps the same shape — the
binary is an external asset, never compiled into the executable.

Resolution order:

1. `vendor/ripgrep/<arch>-<os>/rg` next to the executable (release archives)
2. `vendor/ripgrep/<arch>-<os>/rg` in the source tree (development)
3. the host's `rg` on `PATH`

So either drop a pinned ripgrep under `vendor/ripgrep/`, or just have `rg`
installed. `/doctor` reports which one is in use. Setting
`USE_BUILTIN_RIPGREP=0` forces the host binary.

## Testing

```sh
just test          # full suite through cargo-nextest — the gate
just t <pattern>   # substring filter
just check         # type/borrow check only, links nothing
```

`cargo test` is **not** a valid gate for this crate. It runs the whole suite as
threads in one process, and this codebase has process-wide state (env vars,
`OnceLock` caches) that leaks between tests as a result. `just test` runs under
`cargo-nextest`, which gives each test its own process, and pins the host state
a test would otherwise inherit. The justfile header explains the measurements
behind that.

## License

[AGPL-3.0-only](LICENSE).

Note the network clause: if you run a modified version as a network service,
its users are entitled to the modified source.

[CometixTUI]: https://github.com/Haleclipse/CometixTUI
