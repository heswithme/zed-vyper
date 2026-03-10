# zed-vyper

Reference-quality Vyper support for Zed.

## What It Provides

- Native Zed language support for `.vy` and `.vyi`
- Tree-sitter-based highlighting, outline, indentation, bracket matching, and text objects
- `vyper-lsp` integration for diagnostics, completion, hover, references, and navigation
- User-managed `vyper-lsp` integration with explicit binary override support
- A clean split between parser maintenance in `vyper-tree-sitter` and editor UX in this repo

## Backend Strategy

`zed-vyper` expects users to install `vyper-lsp` themselves.

Syntax support aims to stay broad enough to open real-world Vyper files cleanly.
LSP support is intentionally aligned with the current `vyper-lsp` support window
for modern Vyper projects.

## Grammar Pin

This extension is pinned to:

- grammar repo: `https://github.com/heswithme/vyper-tree-sitter`
- revision: `6c2356f9f855b17c5a9192d8217f7bb0e07c1771`

All Zed-specific `.scm` queries live in this repository under `languages/vyper/`.
The grammar repository is treated as the parser source, not the editor UX layer.

## Install `vyper-lsp`

Recommended:

```bash
uv tool install vyper-lsp
```

The extension resolves `vyper-lsp` in this order:

1. `lsp.vyper-lsp.binary.path` if you configured one explicitly
2. `vyper-lsp` on your `PATH`

If neither exists, Zed shows a language server failure telling you to install
`vyper-lsp` or configure an explicit binary path.

## Zed Configuration

If you already manage `vyper-lsp` yourself, you can point Zed to it explicitly:

```json
{
  "lsp": {
    "vyper-lsp": {
      "binary": {
        "path": "/absolute/path/to/vyper-lsp"
      }
    }
  }
}
```

You can also pass custom arguments, environment variables, initialization
options, and workspace settings:

```json
{
  "lsp": {
    "vyper-lsp": {
      "binary": {
        "path": "/absolute/path/to/vyper-lsp",
        "arguments": [],
        "env": {
          "PATH": "/custom/bin:/usr/bin:/bin"
        }
      },
      "initialization_options": {},
      "settings": {}
    }
  }
}
```

## Development

### Repository Layout

- `extension.toml`: Zed extension manifest, capabilities, and pinned grammar reference
- `Cargo.toml` and `src/lib.rs`: Zed extension runtime for resolving and launching `vyper-lsp`
- `languages/vyper/`: language registration plus Zed Tree-sitter queries
- `fixtures/`: Vyper samples used for smoke coverage of syntax queries

### Local Development

1. Install Rust with `rustup`.
2. Build the extension runtime:

```bash
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/zed_vyper.wasm extension.wasm
```

3. In Zed, run `zed: install dev extension` and point it at this directory.
4. Trust the workspace so Zed can start `vyper-lsp`.
5. Install `vyper-lsp` yourself if you want to test LSP features:

```bash
uv tool install vyper-lsp
```

6. Optionally override the server path through `lsp.vyper-lsp.binary.path`.

### Verification

```bash
cargo check
cargo build --target wasm32-wasip2 --release
```

For grammar smoke checks, parse the fixtures with the pinned grammar revision:

```bash
git clone https://github.com/heswithme/vyper-tree-sitter /tmp/vyper-tree-sitter
git -C /tmp/vyper-tree-sitter checkout 6c2356f9f855b17c5a9192d8217f7bb0e07c1771
cd /tmp/vyper-tree-sitter
tree-sitter generate
ZED_VYPER_DIR=/path/to/zed-vyper
tree-sitter parse -q -p /tmp/vyper-tree-sitter \
  "$ZED_VYPER_DIR"/fixtures/*.vy \
  "$ZED_VYPER_DIR"/fixtures/*.vyi
```

## Scope for v1

Included:

- publishable grammar pin
- syntax highlighting and editor ergonomics
- user-managed `vyper-lsp` integration with explicit binary override support
- Zed-native text objects for functions and declarations

Deferred:

- alternate backends such as `couleuvre`
- compiler orchestration beyond what `vyper-lsp` already provides
- semantic token customization unless the backend starts emitting custom tokens

## Repository

Canonical repository: `https://github.com/heswithme/zed-vyper`
