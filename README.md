# zed-vyper

Reference-quality Vyper support for Zed.

## What It Provides

- Native Zed language support for `.vy` and `.vyi`
- Tree-sitter-based highlighting, outline, indentation, bracket matching, and text objects
- Default `vyper-lsp` integration for diagnostics, completion, hover, references, and navigation
- Optional Couleuvre backend support through the same Zed LSP slot
- User-managed backend integration with explicit binary override support
- Automatic workspace `.venv` exposure for Vyper libraries installed in site-packages
- A clean split between parser maintenance in `vyper-tree-sitter` and editor UX in this repo

## Backend Strategy

`zed-vyper` supports two user-managed backends:

- `vyper-lsp` (default)
- `couleuvre` (optional)

The backend selector lives under `lsp.vyper-lsp.settings.backend`, and defaults
to `vyper-lsp` when unset or invalid.

To help global `vyper-lsp` resolve workspace-installed Vyper libraries, the
extension prepends common `.venv` candidate paths from the current worktree and
its ancestors before launching `vyper-lsp`:

- `.venv/lib/python3.15` through `.venv/lib/python3.10` `site-packages` paths to `PYTHONPATH`
- `.venv/bin` to `PATH`

That `.venv` env shaping is only applied to `vyper-lsp`. Couleuvre keeps its
own documented environment/version model.

Syntax support aims to stay broad enough to open real-world Vyper files cleanly.
`vyper-lsp` remains the recommended default because it is more feature-complete
today. Couleuvre is available as a lighter-weight alternative.

At the time of writing, the published `couleuvre` package is missing a declared
`packaging` dependency and ships a broken console script, so Couleuvre support
in `zed-vyper` currently requires explicit binary configuration.

The working standalone install command is:

```bash
uv tool install --with packaging couleuvre
```

## Grammar Pin

This extension is pinned to:

- grammar repo: `https://github.com/heswithme/vyper-tree-sitter`
- revision: `6f78ae655bc405e2be898e30cf70ff37121fc933`

All Zed-specific `.scm` queries live in this repository under `languages/vyper/`.
The grammar repository is treated as the parser source, not the editor UX layer.

## Install a Backend

Recommended default:

```bash
uv tool install vyper-lsp
```

Optional alternative:

```bash
uv tool install --with packaging couleuvre
```

Couleuvre source-run fallback from a Couleuvre checkout:

```bash
uv sync
uv run --with packaging -m couleuvre
```

Ephemeral fallback from any directory:

```bash
uv run --with couleuvre --with packaging -m couleuvre
```

Backend resolution works like this:

1. `lsp.vyper-lsp.binary.path` if you configured one explicitly
2. for the default backend only, `vyper-lsp` on your `PATH`

If neither exists, Zed shows a language server failure telling you to install
the selected backend or configure an explicit binary path.

If the workspace uses a standard `.venv`, no additional Zed configuration is
needed for libraries installed there when using `vyper-lsp`.
The important path is the `site-packages` directory, not `.venv` itself.

## Zed Configuration

Default backend, explicit binary path:

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

If you installed Couleuvre with:

```bash
uv tool install --with packaging couleuvre
```

the recommended Zed configuration is to point at the tool-env Python and run
the module directly:

```json
{
  "lsp": {
    "vyper-lsp": {
      "settings": {
        "backend": "couleuvre"
      },
      "binary": {
        "path": "/absolute/path/from/uv-tool-dir/couleuvre/bin/python",
        "arguments": ["-m", "couleuvre"]
      }
    }
  }
}
```

`which couleuvre` points at the wrapper script, not the Python interpreter you
should put into Zed settings. To print a ready-to-paste config block from the
installed wrapper, use:

```bash
p="$(head -n1 "$(which couleuvre)" | sed 's/^#!//')" && printf '{\n  "lsp": {\n    "vyper-lsp": {\n      "settings": {\n        "backend": "couleuvre"\n      },\n      "binary": {\n        "path": "%s",\n        "arguments": ["-m", "couleuvre"]\n      }\n    }\n  }\n}\n' "$p"
```

Couleuvre via `uv run` from an arbitrary workspace:

```json
{
  "lsp": {
    "vyper-lsp": {
      "settings": {
        "backend": "couleuvre"
      },
      "binary": {
        "path": "uv",
        "arguments": ["run", "--with", "couleuvre", "--with", "packaging", "-m", "couleuvre"]
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
      "settings": {
        "backend": "vyper-lsp"
      },
      "binary": {
        "path": "/absolute/path/to/vyper-lsp",
        "arguments": [],
        "env": {
          "PATH": "/custom/bin:/usr/bin:/bin"
        }
      },
      "initialization_options": {},
      "settings": {
        "backend": "vyper-lsp"
      }
    }
  }
}
```

`backend` is consumed by the extension and is not forwarded to the selected
language server as workspace configuration.

## Development

### Repository Layout

- `extension.toml`: Zed extension manifest, capabilities, and pinned grammar reference
- `Cargo.toml` and `src/lib.rs`: Zed extension runtime for resolving and launching the selected Vyper backend
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
4. Trust the workspace so Zed can start the selected language server.
5. Install a backend yourself if you want to test LSP features:

```bash
uv tool install vyper-lsp
```

For Couleuvre instead:

```bash
uv tool install --with packaging couleuvre
```

6. Select Couleuvre with `lsp.vyper-lsp.settings.backend = "couleuvre"`.
7. Configure `lsp.vyper-lsp.binary.path` to a working Couleuvre command.
8. If you want to test `vyper-lsp` workspace dependency resolution, create a root-level
   `.venv` and install Vyper libraries there.

### Verification

```bash
cargo check
cargo build --target wasm32-wasip2 --release
```

For grammar smoke checks, parse the fixtures with the pinned grammar revision:

```bash
git clone https://github.com/heswithme/vyper-tree-sitter /tmp/vyper-tree-sitter
git -C /tmp/vyper-tree-sitter checkout 6f78ae655bc405e2be898e30cf70ff37121fc933
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
- user-managed `vyper-lsp` and Couleuvre integration with explicit binary override support
- automatic workspace `.venv` env injection for site-packages imports
- Zed-native text objects for functions and declarations

Deferred:

- compiler orchestration beyond what `vyper-lsp` already provides
- semantic token customization unless the backend starts emitting custom tokens

## Repository

Canonical repository: `https://github.com/heswithme/zed-vyper`
