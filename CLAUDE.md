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

# Run all tests
cargo test

# Run a single test by name
cargo test test_basic_operations
cargo test test_prefix_operations

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

The tool compares two text files line-by-line, finding lines present in one file but absent in the other, independent of line order.

**Data flow in `main.rs`:**
1. Each file is read into a `TernarySearchTrie<u32>` — keys are line strings, values are line numbers (`u32`).
2. The two tries are walked to compute the symmetric difference: lines in file A not in B, and lines in B not in A.
3. Results are sorted by original line number, then rendered as text or HTML.

**`src/trie/ternary_trie.rs`** — the only non-trivial data structure. A generic `TernarySearchTrie<E>` where each node stores a character and branches left (char < node), middle (next char in key), or right (char > node). Supports `put`, `get`, `delete`, `contains`, `get_all_keys`, `get_keys_with_prefix`, and `longest_prefix_of`. The value type `E` must be `Clone`.

**`src/clap_parser/mod.rs`** — CLI args via `clap` derive: `--first` / `-f`, `--second` / `-s`, `--ignore-case` / `-i`, `--render-html` / `-r`.

**Known limitation:** duplicate lines in a file are silently overwritten in the trie — only the last occurrence's line number is kept. This is tracked with a `// TODO` in `main.rs`.

**Test files** live in `src/trie/` alongside the implementation: `tests.rs` (unit), `integration_tests.rs`, and `tests_original.rs`. Sample input data is in `src/test_data/`.
