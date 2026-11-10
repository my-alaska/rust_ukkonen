# Ukkonen's suffix tree construction in Rust

A small library, written as a Rust exercise, that builds suffix trees in linear
time with Ukkonen's algorithm and uses them for substring search.

- Works for any element type `T: Eq + Hash` (characters, bytes, integers, words, ...).
The tree borrows the elements, so nothing is copied.
- The tree is built on-line: `extend_tree` appends one element, `extend` a whole
slice, and the tree can be extended further at any time, also after queries.
- `find` can be called at any time.
- No dependencies.

### The algorithm

Ukkonen's algorithm reads the text and keeps the suffix tree of
the prefix read so far. It runs in O(n).

1. **Open leaf edges**: leaves implicitly point at the end of the ingested sequence.
2. **Active point + remainder**: suffixes that already occured earlier in text
are kept implicit and only inserted once the pattern breaks.
3. **Suffix links**: reference to nodes that look the same. From the node for `xa`
jump straight to the node for `a` instead of walking from the root.

The algorithm implementatin lays in `UkkonenTree::extend_tree` - [src/lib.rs](src/lib.rs).

### Setup

Requires Rust 1.85+ (edition 2024). Install with [rustup](https://rustup.rs/)

```sh
git clone https://github.com/my-alaska/rust_ukkonen.git
cd rust_ukkonen
cargo build # or: cargo build --release
```

### Usage

```toml
[dependencies]
rust_ukkonen = { git = "https://github.com/my-alaska/rust_ukkonen" }
```

```rust
use rust_ukkonen::UkkonenTree;

let text: Vec<char> = "banana".chars().collect();
let mut tree = UkkonenTree::new(&text);

let pattern: Vec<char> = "ana".chars().collect();
assert_eq!(tree.find(&pattern), vec![1, 3]); // positions in ascending order

// The tree remains open: extend it by one element,
let more: Vec<char> = "bandana".chars().collect();
tree.extend_tree(&more[0]);
// or by a whole slice (calls extend_tree for each element)
tree.extend(&more[1..]);
assert_eq!(tree.find(&pattern), vec![1, 3, 10]);

tree.print_tree();
```

### Tests

```sh
cargo test                     # everything except timing
cargo test --release           # everything, including timing
cargo test --test correctness  # one test file
cargo test unary_texts         # tests containing "unary_texts" in their name
cargo test -- --nocapture      # show outputs
```

| File | Checks |
| --- | --- |
| `basic.rs` | small hand-written examples |
| `correctness.rs` | `find` against vs naive search: tricky inputs, thousands of random texts, edge cases, on-line construction, non-`char` types |
| `complexity.rs` | how time and memory grow with input length |
| `common/mod.rs` | shared helpers: deterministic RNG, naive search, input generators |

**Complexity tests** build trees for doubling n and fit `cost ~ c * n^k` on a
log-log scale (`k ~ 1` is linear, `k ~ 2` means an O(n^2) bug). Inputs: random
text over 4 and 26 letters, Fibonacci words and `aaaa...`.

  ```sh
  cargo test --release --test complexity -- --nocapture
  ```

  `--nocapture` prints per-size tables; per-element columns should stay roughly constant.
