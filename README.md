# zed-vyper

Reference-quality Vyper support for Zed.

## What It Provides

- Native Zed language support for `.vy` and `.vyi`
- Tree-sitter-based highlighting, outline, indentation, bracket matching, and text objects
- `vyper-lsp` integration for diagnostics, completion, hover, references, and navigation
- Managed `vyper-lsp` provisioning for the default case, with manual override support
- A clean split between parser maintenance in `vyper-tree-sitter` and editor UX in this repo

## Backend Strategy

`zed-vyper` provisions `vyper-lsp` automatically for the common case.

Syntax support aims to stay broad enough to open real-world Vyper files cleanly.
LSP support is intentionally aligned with the current `vyper-lsp` support window
for modern Vyper projects.

## Grammar Pin

This extension is pinned to:

- grammar repo: `https://github.com/heswithme/vyper-tree-sitter`
- revision: `6c2356f9f855b17c5a9192d8217f7bb0e07c1771`

All Zed-specific `.scm` queries live in this repository under `languages/vyper/`.
The grammar repository is treated as the parser source, not the editor UX layer.

## Managed `vyper-lsp`

On first use, the extension tries the following in order:

1. `lsp.vyper-lsp.binary.path` if you configured one explicitly
2. a cached managed install inside the extension working directory
3. a managed install via `uv` if `uv` is available on your `PATH`
4. a managed install via Python 3.12+ and `venv` if a suitable interpreter is available
5. `vyper-lsp` already installed on your `PATH`

If all of those fail, Zed shows an installation failure for the language server.

The managed install is pinned to:

- `vyper-lsp==0.1.4`

## Zed Configuration

Automatic provisioning is the default UX, but manual configuration is still
supported. If you already manage `vyper-lsp` yourself, you can point Zed to it:

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

## Environment Requirements

For the automatic managed install path, one of these needs to be available:

- `uv`
- Python 3.12+

If neither is present, the extension falls back to a `vyper-lsp` already on
`PATH` or an explicitly configured binary override.

## Development

### Repository Layout

- `extension.toml`: Zed extension manifest, capabilities, and pinned grammar reference
- `Cargo.toml` and `src/lib.rs`: Zed extension runtime for provisioning and launching `vyper-lsp`
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
5. For testing managed install, ensure either `uv` or Python 3.12+ is available.
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
- managed `vyper-lsp` provisioning with explicit binary override support
- Zed-native text objects for functions and declarations

Deferred:

- alternate backends such as `couleuvre`
- compiler orchestration beyond what `vyper-lsp` already provides
- semantic token customization unless the backend starts emitting custom tokens

## Repository

Canonical repository: `https://github.com/heswithme/zed-vyper`
