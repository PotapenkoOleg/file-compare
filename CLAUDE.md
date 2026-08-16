# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Build optimized release binary
cargo build --release

# Run
cargo run -- --first <file1> --second <file2>
cargo run -- --first <file1> --second <file2> --ignore-case
cargo run -- --first <file1> --second <file2> --render-html

# Run all tests (workspace-wide; `default-members` makes plain `cargo test` equivalent)
cargo test --workspace

# Run a single test by name
cargo test -p ternary_trie test_basic_operations
cargo test -p ternary_trie test_prefix_operations

# Lint
cargo clippy --workspace

# Format
cargo fmt
```

## Architecture

The tool compares two text files line-by-line, finding lines present in one file but absent in the other, independent of line order.

**Workspace layout:** a two-member Cargo workspace. The root package `file_compare` is the binary (CLI parsing, file reading, diffing, rendering); `crates/ternary_trie` is a standalone, dependency-free library package holding the trie, consumed as a path dependency. `default-members = [".", "crates/ternary_trie"]` keeps plain `cargo run` and `cargo test` working from the repo root.

**Data flow in `main.rs`:**
1. Each file is read into a `TernarySearchTrie<u32>` — keys are line strings, values are line numbers (`u32`).
2. The two tries are walked to compute the symmetric difference: lines in file A not in B, and lines in B not in A.
3. Results are sorted by original line number, then rendered as text or HTML.

**`crates/ternary_trie/src/symbol_table.rs`** — the traits the trie implements, split so that the map-like half is separable from the trie-specific half:
- `SymbolTable<E: Clone>` — `put`, `get`, `delete`, `contains`, `clear`, `is_empty`, `get_size`, `get_all_keys`. Any keyed store could provide these.
- `PrefixSearch` — `get_keys_with_prefix` and `longest_prefix_of`. These exploit the shared structure of stored keys, which is the reason to pick a trie over a hash map. Note the two searches run in opposite directions: `get_keys_with_prefix` finds stored keys *extending* the argument, `longest_prefix_of` finds the longest stored key *contained in* it.

Both traits are dyn-compatible; the module's doctests pin that, the lexicographic key ordering, and the direction of each prefix query.

**`crates/ternary_trie/src/ternary_trie.rs`** — the only non-trivial data structure. A generic `TernarySearchTrie<E>` where each node stores a character and branches left (char < node), middle (next char in key), or right (char > node). Only `new()` is inherent (plus `Default`, which delegates to it); the ten public operations live in `impl<E: Clone> SymbolTable<E>` and `impl<E: Clone> PrefixSearch` blocks, so **callers must import the trait** — `use ternary_trie::{SymbolTable, TernarySearchTrie};`. The private recursive helpers stay in `impl<E: Clone> TernarySearchTrie<E>`. The value type `E` must be `Clone` because `get` returns an owned copy.

**`src/clap_parser/mod.rs`** — CLI args via `clap` derive: `--first` / `-f`, `--second` / `-s`, `--ignore-case` / `-i`, `--render-html` / `-r`.

`--ignore-case` is implemented by storing `to_uppercase()` of each line as the trie key, so the *original* text is never retained — diff output under this flag prints the uppercased form, not the file's original casing.

**`crates/ternary_trie/src/lib.rs`** — crate root for the trie library: declares the public `symbol_table` and `ternary_trie` submodules, re-exports `SymbolTable`, `PrefixSearch`, and `TernarySearchTrie` at the crate root (so consumers write `use ternary_trie::TernarySearchTrie;` rather than repeating the module name), and declares the three test modules (`tests`, `integration_tests`, `tests_original`). Each test file gates itself with an inner `#[cfg(test)] mod ...` and imports via `use crate::{PrefixSearch, SymbolTable, TernarySearchTrie};`.

**Known limitations:**
- Duplicate lines in a file are silently overwritten in the trie — only the last occurrence's line number is kept. Tracked with a `// TODO` in `main.rs`.
- HTML output is not entity-escaped — lines containing `<`, `>`, `&`, or `"` will malform the table or inject markup when using `--render-html`.
- Reported line numbers are 0-indexed (`enumerate()`), so the first line of a file prints as "line 0".

**Test files** live in `crates/ternary_trie/src/` alongside the implementation: `tests.rs` (unit), `integration_tests.rs`, and `tests_original.rs` — 17 tests total, all exercising only the trie's public API, plus 3 doctests in `symbol_table.rs`. Sample input data for the binary stays in `src/test_data/`.
