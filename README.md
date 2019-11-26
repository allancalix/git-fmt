# git-fmt

git-fmt extends git to easily apply a variety of formatters to a set of changes.
Each formatter will be applied (in order) with the provided command to all
matching file extensions. **Formatters are only applied to staged files to
provide a chance to review changes made.**

The basic workflow anticipated might look something like:
```bash
git add .
git fmt
git diff # Examine changes made by formatters.
git add . && git commit -m 'I like the formatter changes!'
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

## Configuration

git-fmt is supported by the standard `git config` options. This means you can
define formatters for all your projects by modifying your global
`.gitconfig` or apply formatter commands only to a specific repository by
modifying the `.git/config` file found in every repository.

You could apply changes by using the cli. For example...
```bash
git config fmt.rust.command "rustfmt {{STAGED_FILE}}" # Apply locally.
git config --global fmt.rust.command "rustfmt {{STAGED_FILE}}"
```

You could also apply changes directly to your global `gitconfig` file.
```yaml
[fmt "rust"]
  command = "rustfmt {{STAGED_FILE}}"
  extensions = "rs"

[fmt "go"]
  command = "gofmt -w {{STAGED_FILE}}"
  extensions = "go, gox" # Separate additional extensions with commas.
```
