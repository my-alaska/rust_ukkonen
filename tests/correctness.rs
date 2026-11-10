//! Correctness tests: the suffix tree must report exactly the same match
//! positions as a naive substring scan.

mod common;

use common::{Rng, all_substrings, chars, fibonacci_text, naive_find, random_text};
use rust_ukkonen::UkkonenTree;

/// Builds a tree over `seq` and checks `find` against the oracle for every
/// distinct substring, plus a few patterns that do not occur.
fn assert_matches_oracle(seq: &[char]) {
    let tree = UkkonenTree::new(seq);

    for pattern in all_substrings(seq) {
        assert_eq!(
            tree.find(&pattern),
            naive_find(seq, &pattern),
            "text {:?}, pattern {:?}",
            String::from_iter(seq),
            String::from_iter(&pattern),
        );
    }

    let absent: [&[char]; 3] = [&['#'], &['a', '#'], &['#', '$']];
    for pattern in absent {
        assert!(
            tree.find(pattern).is_empty(),
            "text {:?}, absent pattern {pattern:?}",
            String::from_iter(seq),
        );
    }
}

// ---------------------------------------------------------------------------
// Hand-picked inputs. Most of them exercise a specific part of Ukkonen's
// algorithm: edge splits, suffix links, walk-down (skip/count), and long
// stretches where `remainder` keeps growing.
// ---------------------------------------------------------------------------

#[test]
fn classic_examples() {
    for text in [
        "banana",
        "mississippi",
        "abcabxabcd", // chain of splits linked by suffix links
        "xabxac",     // Gusfield's textbook example
        "abab",
        "cdddcdc",
        "dedododeeodo", // breaks implementations with a wrong walk-down
        "abcdefabxybcdmnabcdex",
        "babcababx",
        "baaabaaa",
    ] {
        assert_matches_oracle(&chars(text));
    }
}

#[test]
fn single_character_text() {
    assert_matches_oracle(&chars("a"));
}

#[test]
fn unary_texts() {
    // "aaaa...": remainder grows on every step, then everything is resolved at once.
    for len in 1..=40 {
        assert_matches_oracle(&vec!['a'; len]);
    }
}

#[test]
fn all_distinct_characters() {
    // No repeats at all: every phase only inserts leaves at the root.
    let text: Vec<char> = ('a'..='z').collect();
    assert_matches_oracle(&text);
}

#[test]
fn periodic_texts() {
    for (period, reps) in [("ab", 15), ("abc", 10), ("aab", 10), ("abaab", 6)] {
        assert_matches_oracle(&chars(&period.repeat(reps)));
    }
}

#[test]
fn fibonacci_words() {
    for len in [5, 13, 34, 55, 89] {
        assert_matches_oracle(&fibonacci_text(len));
    }
}

// ---------------------------------------------------------------------------
// Randomised differential testing against the naive oracle.
// Small alphabets produce many repeats and therefore many edge splits.
// ---------------------------------------------------------------------------

#[test]
fn random_texts_small_alphabets() {
    let mut rng = Rng::new(42);
    for alphabet in [1, 2, 3, 4] {
        for len in 1..=40 {
            for _ in 0..5 {
                assert_matches_oracle(&random_text(&mut rng, len, alphabet));
            }
        }
    }
}

#[test]
fn random_long_texts_random_patterns() {
    // Too long for all_substrings, so sample patterns: substrings of the text
    // (guaranteed hits) and random strings (mostly misses).
    let mut rng = Rng::new(7);
    for alphabet in [2, 4, 26] {
        let seq = random_text(&mut rng, 2_000, alphabet);
        let tree = UkkonenTree::new(&seq);

        for _ in 0..300 {
            let start = rng.below(seq.len());
            let len = 1 + rng.below(20.min(seq.len() - start));
            let pattern = &seq[start..start + len];
            assert_eq!(tree.find(pattern), naive_find(&seq, pattern));

            let pattern_len = 1 + rng.below(8);
            let random_pattern = random_text(&mut rng, pattern_len, alphabet);
            assert_eq!(
                tree.find(&random_pattern),
                naive_find(&seq, &random_pattern)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Edge cases of `find`.
// ---------------------------------------------------------------------------

#[test]
fn whole_text_is_found_once() {
    let seq = chars("mississippi");
    let tree = UkkonenTree::new(&seq);
    assert_eq!(tree.find(&seq), vec![0]);
}

#[test]
fn pattern_longer_than_text() {
    let seq = chars("abc");
    let tree = UkkonenTree::new(&seq);
    assert!(tree.find(&chars("abcabc")).is_empty());
}

#[test]
fn pattern_diverging_in_the_middle_of_an_edge() {
    let seq = chars("abcdabce");
    let tree = UkkonenTree::new(&seq);
    assert!(tree.find(&chars("abcx")).is_empty());
    assert!(tree.find(&chars("bcdx")).is_empty());
}

#[test]
fn empty_pattern_matches_every_position() {
    // Every suffix starts with the empty string.
    let seq = chars("banana");
    let tree = UkkonenTree::new(&seq);
    assert_eq!(tree.find(&[]), naive_find(&seq, &[]));
}

#[test]
fn empty_text() {
    let tree = UkkonenTree::<char>::new(&[]);
    assert!(tree.find(&chars("a")).is_empty());
    assert!(tree.find(&[]).is_empty());
}

// ---------------------------------------------------------------------------
// Online construction.
// ---------------------------------------------------------------------------

#[test]
fn online_construction_matches_batch_construction() {
    let mut rng = Rng::new(1234);
    for _ in 0..50 {
        let seq = random_text(&mut rng, 60, 3);

        let batch = UkkonenTree::new(&seq);
        let mut online = UkkonenTree::new(&[]);
        for element in &seq {
            online.extend_tree(element);
        }

        for pattern in all_substrings(&seq) {
            assert_eq!(online.find(&pattern), batch.find(&pattern));
        }
    }
}

#[test]
fn extend_with_a_slice() {
    let start = chars("abra");
    let rest = chars("cadabra");
    let mut tree = UkkonenTree::new(&start);
    tree.extend(&rest);
    assert_eq!(tree.find(&chars("abra")), vec![0, 7]);
    assert_eq!(tree.find(&chars("a")), vec![0, 3, 5, 7, 10]);
}

#[test]
fn extending_after_queries() {
    let mut rng = Rng::new(2718);
    let chunks: Vec<Vec<char>> = (0..5).map(|_| random_text(&mut rng, 30, 3)).collect();

    let mut tree = UkkonenTree::new(&[]);
    let mut text = Vec::new();
    for chunk in &chunks {
        tree.extend(chunk);
        text.extend_from_slice(chunk);
        for pattern in all_substrings(&text[text.len() - chunk.len()..]) {
            assert_eq!(tree.find(&pattern), naive_find(&text, &pattern));
        }
    }
}

#[test]
fn searchable_after_every_extension() {
    // Prefixes leave some suffixes implicit; `find` must still be exact.
    let mut rng = Rng::new(99);
    for alphabet in [1, 2, 3] {
        for _ in 0..10 {
            let text = random_text(&mut rng, 30, alphabet);
            let mut tree = UkkonenTree::new(&[]);
            for (len, element) in text.iter().enumerate() {
                tree.extend_tree(element);
                let prefix = &text[..=len];
                for pattern in all_substrings(prefix) {
                    assert_eq!(
                        tree.find(&pattern),
                        naive_find(prefix, &pattern),
                        "prefix {:?}, pattern {:?}",
                        String::from_iter(prefix),
                        String::from_iter(&pattern),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generic element types.
// ---------------------------------------------------------------------------

#[test]
fn works_on_integers() {
    let mut rng = Rng::new(5);
    let seq: Vec<u32> = (0..500).map(|_| rng.below(3) as u32).collect();
    let tree = UkkonenTree::new(&seq);
    for pattern in all_substrings(&seq[..40]) {
        assert_eq!(tree.find(&pattern), naive_find(&seq, &pattern));
    }
}

#[test]
fn works_on_words() {
    let seq: Vec<&str> = "to be or not to be that is the question"
        .split(' ')
        .collect();
    let tree = UkkonenTree::new(&seq);
    assert_eq!(tree.find(&["to", "be"]), vec![0, 4]);
    assert_eq!(tree.find(&["be", "that"]), vec![5]);
    assert!(tree.find(&["be", "to"]).is_empty());
}

// ---------------------------------------------------------------------------
// Large inputs.
// ---------------------------------------------------------------------------

#[test]
fn deep_tree_does_not_overflow_stack() {
    // "aaa...ab" builds a chain of internal nodes as deep as the text.
    let n = 100_000;
    let mut seq = vec!['a'; n];
    seq.push('b');
    let tree = UkkonenTree::new(&seq);
    assert_eq!(tree.find(&vec!['a'; n - 1]), vec![0, 1]);
    assert_eq!(tree.find(&['a']).len(), n);
}

#[test]
fn long_implicit_tail() {
    // In "aaa...a" every suffix but the first is implicit.
    let n = 100_000;
    let seq = vec!['a'; n];
    let tree = UkkonenTree::new(&seq);
    assert_eq!(tree.find(&vec!['a'; n - 1]), vec![0, 1]);
    assert_eq!(tree.find(&vec!['a'; n / 2]).len(), n / 2 + 1);
    assert_eq!(tree.find(&['a']).len(), n);
}
