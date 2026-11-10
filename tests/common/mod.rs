//! Test helpers: a deterministic PRNG, input generators and a naive
//! substring search used as the reference.

#![allow(dead_code)] // not every test binary uses every helper

/// Deterministic xorshift64* generator.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // A zero state would make xorshift return zeros forever.
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Value in `0..bound`.
    pub fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

/// Random text of length `len` over the first `alphabet_size` lowercase letters.
pub fn random_text(rng: &mut Rng, len: usize, alphabet_size: usize) -> Vec<char> {
    assert!((1..=26).contains(&alphabet_size));
    (0..len)
        .map(|_| (b'a' + rng.below(alphabet_size) as u8) as char)
        .collect()
}

/// Fibonacci word: f(1) = "b", f(2) = "a", f(n) = f(n-1) + f(n-2), truncated to `len`.
pub fn fibonacci_text(len: usize) -> Vec<char> {
    let (mut prev, mut cur) = (vec!['b'], vec!['a']);
    while cur.len() < len {
        let next = [cur.as_slice(), prev.as_slice()].concat();
        prev = cur;
        cur = next;
    }
    cur.truncate(len);
    cur
}

/// `s` as a `Vec<char>`, the element type most tests use.
pub fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// Reference implementation: every position where `pattern` occurs in `text`.
/// The empty pattern occurs at every position.
pub fn naive_find<T: PartialEq>(text: &[T], pattern: &[T]) -> Vec<usize> {
    if pattern.is_empty() {
        return (0..text.len()).collect();
    }
    text.windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(i, _)| i)
        .collect()
}

/// All distinct substrings of `text` (O(n^2) of them - use on short texts only).
pub fn all_substrings<T: Clone + Ord>(text: &[T]) -> Vec<Vec<T>> {
    let mut subs: Vec<Vec<T>> = (0..text.len())
        .flat_map(|i| (i + 1..=text.len()).map(move |j| text[i..j].to_vec()))
        .collect();
    subs.sort();
    subs.dedup();
    subs
}
