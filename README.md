# herdr-git-graph

A read-only git commit graph TUI for [Herdr](https://herdr.dev). Opens in a split pane beside your work, shows an ASCII branch graph with commit details and diffs.

## Features

- ASCII commit graph in any terminal (no image protocol required)
- Browse commits with keyboard navigation
- View full commit metadata and patch diffs
- Filter by branch (all, local, or a specific branch)
- Fuzzy search commits by subject, author, or hash
- Read-only — never modifies your repository

## Requirements

- [Herdr](https://herdr.dev) 0.7.0+
- Git on `PATH`
- Optional: [delta](https://github.com/dandavison/delta) for styled diffs
- Rust 1.70+ only when a prebuilt binary is unavailable for your platform

## Install and run

Install the plugin from this public GitHub repository:

```bash
herdr plugin install jorge-huxley/herdr-git-graph
```

Herdr shows the plugin manifest and commands for review before installation. The
installer downloads a SHA-256-verified prebuilt binary when one is available and
otherwise builds from source with Cargo.

Run it directly from a Herdr session:

```bash
# Linux and macOS
herdr plugin action invoke open-git-graph --plugin herdr-git-graph

# Windows
herdr plugin action invoke open-git-graph-windows --plugin herdr-git-graph
```

For frequent use, add one or both actions to your Herdr `config.toml`. Run
`herdr --help` to see its location on your platform.

```toml
# Open in a split pane (Linux and macOS).
[[keys.command]]
key = "prefix+g"
type = "shell"
command = "herdr plugin action invoke open-git-graph --plugin herdr-git-graph"

# Open in its own tab (Linux and macOS).
[[keys.command]]
key = "prefix+shift+g"
type = "shell"
command = "herdr plugin action invoke open-git-graph-tab --plugin herdr-git-graph"
```

On Windows, use `open-git-graph-windows` and
`open-git-graph-tab-windows` instead. Then reload the configuration:

```bash
herdr config check
herdr server reload-config
```

Open a Git repository in Herdr and press the configured key. Invoking the same
action again focuses the existing graph; invoking it while focused closes it.

## Local development

```bash
herdr plugin link /path/to/herdr-git-graph
cargo build --release
```

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` or arrows | Move selection |
| `Enter` / `d` | Toggle diff view |
| `b` | Branch filter picker |
| `/` | Search commits |
| `Ctrl-u` / `Ctrl-d` | Scroll details/diff |
| `?` | Help |
| `q` / `Esc` | Close |

## Configuration

Copy [`config.example.toml`](config.example.toml) to the directory from:

```bash
herdr plugin config-dir herdr-git-graph
```

## Marketplace

To list this plugin on [herdr.dev/plugins](https://herdr.dev/plugins), add the GitHub topic `herdr-plugin` to your public repository.

## License

MIT
