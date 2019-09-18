# git-fmt

git-fmt extends git to easily apply a variety of formatters to a set of changes.
Formatters are configured in a `GitFormat.toml` located in the root of the
repository.

Example `./GitFormat.toml`
```toml
[rust]
  command = "rustfmt {{STAGED_FILE}}"
  extensions = ["rs"]

[go]
  command = "gofmt {{STAGED_FILE}} > {{STAGED_FILE}}"
  extensions = ["go"]
```

Formatting is _only_ applied to files that are staged. This makes it easy to
verify changes before commit. Files that are staged and also modified will be
ignored to prevent unverifiable changes.

## Use
```bash
# From anywhere in the repository
git fmt
```

## Installation

1. `git-fmt` is a standalone binary. To install, you can either build from source
or download the (OSX only) binary from
[releases](https://github.com/allancalix/git-fmt/releases).

  ### Building from source
  ```bash
    git clone https://github.com/allancalix/git-fmt.git && cd git-fmt
    cargo build --release
  ```
2. Place the binary somewhere in `$PATH`.

