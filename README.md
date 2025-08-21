# srt2txt

Convert SRT subtitle files into clean plain text. Fast & parallel with safe defaults and composable post‑processing.

## Features
* Removes numbering & timestamps
* Strips simple HTML tags (`<i>`, `<b>`, etc.)
* Joins multi‑line caption blocks into single logical lines
* Join multiple files into one (`--join`) OR flatten all captions into one sentence stream (`--join-sentences`)
* Parallel file processing via Rayon
* Optional immediate duplicate removal & blank line collapsing
* Refuses overwrites unless `--force`

## Install
```bash
cargo install srt2txt
```

## Quick Start
```bash
# Basic conversion (creates input.txt next to input.srt)
srt2txt input.srt

# Convert a directory tree recursively → outputs into cleaned/
srt2txt subtitles/ --output-dir cleaned/

# Join several files into combined.txt (default name)
srt2txt a.srt b.srt --join

# Custom join filename
srt2txt a.srt b.srt --join --join-name all_dialogue.txt

# Stream per‑file results to stdout (no files written)
srt2txt movie.srt --stdout

# Single continuous text block (no blank separators)
srt2txt movie.srt --join-sentences --stdout
```

## Sample Input / Output
Input (`sample.srt`):
```
1
00:00:00,000 --> 00:00:01,000
<i>Hello</i>  world!

2
00:00:01,500 --> 00:00:03,000
Second  line.
Line continued.
```
Output (`sample.txt`):
```
Hello world!

Second line. Line continued.
```

## Flags
```
--stdout              Print combined per-file outputs to stdout (disables file writes)
--join                Aggregate all processed files into a single output (excludes --stdout, --join-sentences)
--join-name <NAME>    Filename for --join output (default: combined.txt)
--join-sentences      Flatten all captions into one continuous block (ignores --collapse-blank; conflicts with --join)
--collapse-blank      Collapse ≥3 consecutive newlines to a single blank line
--remove-duplicates   Remove immediately repeated normalized lines
-f, --force           Overwrite existing output files
-o, --output-dir DIR  Directory for outputs (created if missing)
```

### Flag Interaction Matrix
| Combination | Allowed | Notes |
|-------------|---------|-------|
| `--stdout` + `--join` | ❌ | Mutually exclusive |
| `--join` + `--join-sentences` | ❌ | Different aggregation semantics |
| `--join-sentences` + `--collapse-blank` | ⚠️ | Runs; blank collapse skipped (warning emitted) |
| Existing output & no `--force` | ❌ | Aborts to prevent overwrite |

## Edge Cases & Behavior
* Duplicate sequence numbers: accepted if timestamps parse.
* Malformed timestamp lines: block skipped (robust > strict).
* Non-`.srt` files ignored; directories walked recursively.
* HTML stripping is naive (`<[^>]+>`): removes any simple tag.
* Dedup removes only immediately consecutive identical cleaned lines.

## Performance Notes
* File-level parallelism only (predictable memory & ordering).
* Regex patterns compiled once via `lazy_static`.
* Per-file processing builds a vector of cleaned caption strings; large joined output materialized only at final aggregation.

## Development
Core logic resides in `src/main.rs` (parsing, cleaning, post-process, CLI validation).

Build & test locally:
```bash
cargo build
cargo test --all --quiet
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Git Hooks (cargo-husky)
Installed/updated on `cargo build` according to `Cargo.toml`:

| Hook | Command(s) |
|------|-----------|
| pre-commit | `cargo fmt --all -- --check` then `cargo clippy --all-targets --all-features -- -D warnings` |
| pre-push | `cargo test --all --quiet` |

Skip temporarily:
```bash
HUSKY=0 git commit -m "wip"
```

Force regenerate if stale:
```bash
cargo clean -p cargo-husky || true
cargo build
```

## Contributing
Keep features lean. Add unit tests beside changed logic (see existing tests in `src/main.rs`). Maintain flag validation in `main`. When modularizing, prefer pure helpers (e.g. `parser.rs`, `transform.rs`) while preserving existing behavior.

## License
MIT
