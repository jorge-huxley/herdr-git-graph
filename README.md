# herdr-git-graph

A read-only git commit graph TUI for [Herdr](https://herdr.dev). Opens in a split
pane beside your work and shows a colored ASCII branch graph with ref labels,
dates, authors, and on-demand commit details or diffs.

## Features

- Colored ASCII commit graph with per-lane branch colors (no image protocol required)
- Branch, remote, tag, and `HEAD` labels shown as colored pills on each commit
- Commit list includes relative date, author, and short hash
- Commit details and patch diffs open only when requested (`Enter` / `d`)
- Browse commits with keyboard navigation
- Filter by branch (all, local, or a specific branch)
- Fuzzy search commits by subject, author, hash, or ref name
- `q` / `Esc` quit closes the Herdr host pane (no leftover empty shell pane)
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
Quitting from inside the TUI with `q` (or `Esc` when the details pane is already
closed) also closes the Herdr pane.

## Local development

```bash
herdr plugin link /path/to/herdr-git-graph
cargo build --release
```

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` or arrows | Move selection |
| `PageUp` / `PageDown` | Jump selection by 10 commits |
| `Enter` | Open commit details pane |
| `d` | Open commit diff (again switches back to details) |
| `b` | Branch filter picker |
| `/` | Search commits |
| `Ctrl-u` / `Ctrl-d` | Scroll details/diff |
| `?` | Help |
| `Esc` | Close details/dialog, or quit and close Herdr pane |
| `q` | Quit and close Herdr pane |

The graph starts full-width. The right-hand commit pane appears only after
`Enter` (details) or `d` (diff).

## Configuration

Copy [`config.example.toml`](config.example.toml) to the directory from:

```bash
herdr plugin config-dir herdr-git-graph
```

## Marketplace

To list this plugin on [herdr.dev/plugins](https://herdr.dev/plugins), add the GitHub topic `herdr-plugin` to your public repository.

## License

MIT
