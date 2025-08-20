# srt2txt

Convert SRT subtitle files into clean plain text.

## Features
- Removes numbering & timestamps
- Strips simple HTML tags (<i>, <b>, etc.)
- Joins multi-line captions into single lines
- Optionally joins multiple files into one output
- Parallel processing of many files
- Optional duplicate removal and blank line collapsing
- Optionally join all captions into continuous sentences (`--join-sentences`)

## Install
```bash
cargo install --path .
```

## Usage
```bash
srt2txt input.srt
srt2txt subtitles/ --output-dir cleaned/
srt2txt a.srt b.srt --join --join-name all.txt
srt2txt movie.srt --stdout
srt2txt movie.srt --join-sentences --stdout
```

Flags:
```
--stdout              Print combined output of all inputs to stdout
--join                Join all processed captions into a single output file
--join-name <NAME>    Output file name when using --join (default: combined.txt)
--join-sentences      Output a single continuous text block (no blank lines)
--collapse-blank      Collapse multiple blank lines into one
--remove-duplicates   Remove immediately repeated lines
-f, --force           Overwrite existing output files
-o, --output-dir DIR  Output directory (created if missing)
```

## License
MIT

## Development
### Git Hooks
Git hooks are managed automatically by [cargo-husky](https://crates.io/crates/cargo-husky). They install on build.

After cloning (or after changing `Cargo.toml` hook config):
```bash
cargo build  # generates/updates hooks in .git/hooks
```

Current configured hooks (from `Cargo.toml`):

| Hook       | Command(s) |
|------------|------------|
| pre-commit | `cargo fmt --all -- --check` then `cargo clippy --all-targets --all-features -- -D warnings` |
| pre-push   | `cargo test --all --quiet` |

Note: The generated scripts may order clippy/fmt internally (husky may split combined lines). Both checks still run before the commit is accepted.

### Skipping hooks
Temporarily bypass (use sparingly):
```bash
HUSKY=0 git commit -m "wip"
```

### Modifying hooks
Edit the section in `Cargo.toml`:
```toml
[package.metadata.husky]
pre-commit = "cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings"
pre-push = "cargo test --all --quiet"
```
Then rebuild:
```bash
cargo build
```

### Regenerating if stale
If a hook didn’t update, a clean build forces regeneration:
```bash
cargo clean -p cargo-husky || true
cargo build
```

### Disabling permanently (not recommended)
Remove the `cargo-husky` dev-dependency and the `[package.metadata.husky]` section, then delete the hook scripts in `.git/hooks/`.
