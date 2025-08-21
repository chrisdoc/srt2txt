# AI Coding Agent Guide for `srt2txt`

Purpose: High‑performance CLI that converts `.srt` subtitle files into clean plain text with optional joining / sentence flattening / de‑dup / blank line collapsing. Single binary; all logic is currently in `src/main.rs`.

## Architecture & Data Flow
1. Collect inputs (`Cli.inputs`) → expand directories recursively via `walkdir` in `gather_input_files` (filters by `.srt`, case‑insensitive).
2. For each file (Rayon parallel iteration): `parse_srt` → vector of `Caption { start_ms, end_ms, text }`.
3. `clean_text` strips HTML‐like tags (`<...>` regex), normalizes whitespace & joins multiline caption lines into one sentence.
4. Post-processing path chosen by flags:
   - `--join-sentences`: flatten all caption texts into one continuous block (`join_sentences`). Ignores `--collapse-blank` (warning emitted) and disallows `--join`.
   - Per‑file mode: `post_process` (optional dedup + blank collapse) then write adjacent `.txt` or into `--output-dir`.
   - `--join`: aggregate all processed file results then single output file `--join-name` (default `combined.txt`). Mutually exclusive with `--stdout` & `--join-sentences`.
5. Output routing: `--stdout` prints combined per‑file outputs (not join) to STDOUT; otherwise writes files (refuses overwrite unless `--force`).

## Flag Interaction Rules (enforced in `main`)
- Required: at least one input path.
- Mutually exclusive: `--stdout` vs `--join` ; `--join-sentences` vs `--join`.
- `--join-sentences` + `--collapse-blank` => warning (blank collapse skipped by design, since sentence mode removes blank separators).
- Overwrite protection unless `--force`.

## Style & Conventions
- Single-file implementation acceptable for now; if adding modules keep parsing & text transforms stateless and pure where possible.
- Prefer small pure helpers returning `Result<T>` with `anyhow` for context (`with_context` when I/O or parsing could fail).
- Regexes & compiled patterns behind `lazy_static!` to avoid recompilation.
- Parallelism: file-level only (Rayon `par_iter`)—avoid interior mutability; return owned `String`s then combine.
- Deduplication only removes immediately repeated (normalized) lines; preserve ordering.

## Adding Features Safely
- Maintain flag exclusivity logic near existing checks in `main`; emit clear error via `anyhow::bail!` for invalid combos.
- When introducing new text transforms: integrate after `parse_srt` but before aggregation; keep `join_sentences` semantics (continuous text without blank lines) consistent.
- If adding output formats, branch after current write logic (`write_output`) rather than altering parsing.

## Testing Approach
- Existing tests colocated in `src/main.rs` under `#[cfg(test)]`:
  - Timestamp parsing edge cases (`parse_timestamp`).
  - Parsing + cleaning HTML tag removal & whitespace normalization.
  - Post-processing dedup behavior.
- For new parsing rules, add focused unit tests (temporary file with crafted SRT) rather than large integration harness.

## Developer Workflows
- Build: `cargo build` (also installs/updates Git hooks via `cargo-husky`).
- Format & Lint (CI parity): `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Test: `cargo test --all --quiet`.
- Install locally: `cargo install --path .` then run `srt2txt ...`.
- Skip hooks for an emergency commit: `HUSKY=0 git commit -m "msg"`.

## Common Edge Cases Handled
- Duplicate sequence numbers tolerated if timestamps valid.
- Malformed timestamp lines skipped silently for that block.
- Non-SRT extensions ignored; directories traversed recursively.
- HTML tag stripping is naive (`<[^>]+>`); nested/complex tags will be removed wholesale.

## Extension Ideas (Keep Lean)
Document only if implemented—avoid speculative bulk refactors. If splitting files, start with `parser.rs` (SRT parsing) and `transform.rs` (post-processing) while preserving current public function contracts.

(End of file)