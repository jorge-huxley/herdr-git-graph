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

## Install

```bash
herdr plugin install jorge-huxley/herdr-git-graph
```

For local development:

```bash
herdr plugin link /path/to/herdr-git-graph
cargo build --release
```

## Keybinding

Add to `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+g"
type = "shell"
command = "herdr plugin action invoke open-git-graph --plugin herdr-git-graph"

[[keys.command]]
key = "prefix+shift+g"
type = "shell"
command = "herdr plugin action invoke open-git-graph-tab --plugin herdr-git-graph"
```

On Windows, bind the `-windows` action ids (`open-git-graph-windows`, `open-git-graph-tab-windows`).

Run `herdr server reload-config`, then press your key.

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
