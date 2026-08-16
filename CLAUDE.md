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

**`crates/ternary_trie/src/ternary_trie/mod.rs`** — a directory module, so submodules of the trie go in `crates/ternary_trie/src/ternary_trie/` next to it. Holds the only non-trivial data structure. A generic `TernarySearchTrie<E>` where each node stores a character and branches left (char < node), middle (next char in key), or right (char > node). Only `new()` is inherent (plus `Default`, which delegates to it); the ten public operations live in `impl<E: Clone> SymbolTable<E>` and `impl<E: Clone> PrefixSearch` blocks, so **callers must import the trait** — `use ternary_trie::{SymbolTable, TernarySearchTrie};`. The private recursive helpers stay in `impl<E: Clone> TernarySearchTrie<E>`. The value type `E` must be `Clone` because `get` returns an owned copy.

**`src/clap_parser/mod.rs`** — CLI args via `clap` derive: `--first` / `-f`, `--second` / `-s`, `--ignore-case` / `-i`, `--render-html` / `-r`.

`--ignore-case` is implemented by storing `to_uppercase()` of each line as the trie key, so the *original* text is never retained — diff output under this flag prints the uppercased form, not the file's original casing.

**`crates/ternary_trie/src/patricia_trie/mod.rs`** — a self-contained directory module: the implementation plus its own `tests.rs` submodule, declared from `mod.rs` rather than from the crate root. A second implementation of both traits: `PatriciaTrie<E>`, a radix trie storing a run of characters per edge with single-child nodes merged, so a key branches only where it diverges from its neighbours. `Node` children live in a `BTreeMap<char, Node<E>>` keyed by each label's first character, which is what makes traversal lexicographic. `put` splits an edge where a key diverges; `delete` restores the invariant on the way back up by dropping valueless leaves and merging valueless single-child nodes. `PrefixSearch` is implemented without a `Clone` bound here (its helpers never touch values), a weaker requirement than the ternary trie's.

Note the crate is named `ternary_trie` but now holds two independent implementations — the name is narrower than its contents.

**`crates/ternary_trie/src/lib.rs`** — crate root for the trie library: declares the public `patricia_trie`, `symbol_table`, and `ternary_trie` submodules, re-exports `SymbolTable`, `PrefixSearch`, `TernarySearchTrie`, and `PatriciaTrie` at the crate root (so consumers write `use ternary_trie::TernarySearchTrie;` rather than repeating the module name), and declares the three crate-root test modules (`tests`, `integration_tests`, `tests_original`). The Patricia tests are *not* declared here — they belong to `patricia_trie` and are declared from its `mod.rs`. Each test file gates itself with an inner `#[cfg(test)] mod ...` and imports via `use crate::{PrefixSearch, SymbolTable, TernarySearchTrie};` — a crate-root path, so it works from any nesting depth.

**Known limitations:**
- Duplicate lines in a file are silently overwritten in the trie — only the last occurrence's line number is kept. Tracked with a `// TODO` in `main.rs`.
- HTML output is not entity-escaped — lines containing `<`, `>`, `&`, or `"` will malform the table or inject markup when using `--render-html`.
- Reported line numbers are 0-indexed (`enumerate()`), so the first line of a file prints as "line 0".
- `TernarySearchTrie::longest_prefix_of` is not multi-byte safe: it counts matched *characters* but then slices the query by *byte* offset (`prefix[0..length]` in `ternary_trie/mod.rs`). On non-ASCII input it silently truncates (`longest_prefix_of("héllo there")` with `"héllo"` stored returns `Some("héll")`) or panics outright when the offset lands inside a character. `PatriciaTrie` rebuilds the result from characters and is unaffected, so the two disagree here — which is why `test_matches_ternary_search_trie` uses ASCII inputs only. Fix is to collect from the char vector rather than byte-slice.

**Test files** follow two conventions, both with the inner-module gate. The ternary trie's tests sit at the crate root in `crates/ternary_trie/src/` — `tests.rs` (unit), `tests_integration`, `tests_original.rs` — while the Patricia trie's live inside its own module at `patricia_trie/tests.rs`. 24 tests total, all exercising only the public API, plus 4 doctests across `symbol_table.rs` and `patricia_trie/mod.rs`. Sample input data for the binary stays in `src/test_data/`.

Note both layers of every test path are named `tests` for the Patricia module, so its tests report as `patricia_trie::tests::tests::<name>`; the crate-root ones report as `tests::tests::<name>`. Filtering by bare test name (`cargo test test_basic_operations`) matches both implementations' copies at once, which is usually what you want.

`patricia_trie/tests.rs` ends with `test_matches_ternary_search_trie`, which drives both implementations through the same puts, queries, deletes, and a clear via a generic `snapshot<T: SymbolTable<i32> + PrefixSearch + Default>` helper and asserts the two logs are equal. That is the check that keeps the two interchangeable — extend the word/query lists there rather than duplicating assertions per type.
