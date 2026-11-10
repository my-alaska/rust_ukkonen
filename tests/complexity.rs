//! Empirical complexity checks.
//!
//! Ukkonen's algorithm should take O(n) time and memory. We measure the cost
//! for doubling n and fit `cost ~ c * n^k` on a log-log scale: `k ~ 1` is
//! linear, an O(n^2) bug gives `k ~ 2`.
//!
//! * Memory: counted exactly by a per-thread counting allocator, so results
//!   are deterministic and the test runs with plain `cargo test`.
//! * Time: meaningful only for optimised code, so the test is ignored in
//!   debug builds. Run it with
//!   `cargo test --release --test complexity -- --nocapture`.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::time::{Duration, Instant};

use common::{Rng, fibonacci_text, random_text};
use rust_ukkonen::UkkonenTree;

// ---------------------------------------------------------------------------
// Counting allocator
// ---------------------------------------------------------------------------

struct CountingAllocator;

thread_local! {
    // `const` initialisation: accessing these never allocates, which matters
    // because they are touched from inside the allocator itself.
    static LIVE_BYTES: Cell<isize> = const { Cell::new(0) };
    static PEAK_BYTES: Cell<isize> = const { Cell::new(0) };
}

/// Adds `delta` bytes to the live count and updates the peak.
fn record(delta: isize) {
    // `try_with` fails only during thread teardown; ignoring that is fine.
    let _ = LIVE_BYTES.try_with(|live| {
        let now = live.get() + delta;
        live.set(now);
        let _ = PEAK_BYTES.try_with(|peak| peak.set(peak.get().max(now)));
    });
}

// SAFETY: every call is forwarded unchanged to the system allocator;
// we only add bookkeeping around it.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record(layout.size() as isize);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        record(-(layout.size() as isize));
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            record(new_size as isize - layout.size() as isize);
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn live_bytes() -> isize {
    LIVE_BYTES.with(Cell::get)
}

fn reset_peak() {
    let now = live_bytes();
    PEAK_BYTES.with(|peak| peak.set(now));
}

fn peak_bytes() -> isize {
    PEAK_BYTES.with(Cell::get)
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

struct Measurement {
    n: usize,
    time: Duration,
    /// Bytes still owned by the tree once construction finished.
    retained: isize,
    /// Highest allocation level reached during construction.
    peak: isize,
    /// Bytes not returned after the tree was dropped (should be 0).
    leaked: isize,
}

/// Builds the tree once and reports time and memory figures relative to the
/// state before construction (so the input itself is not counted).
fn measure_build(seq: &[char]) -> Measurement {
    let base = live_bytes();
    reset_peak();

    let start = Instant::now();
    let tree = black_box(UkkonenTree::new(black_box(seq)));
    let time = start.elapsed();

    let retained = live_bytes() - base;
    let peak = peak_bytes() - base;
    drop(tree);
    let leaked = live_bytes() - base;

    Measurement {
        n: seq.len(),
        time,
        retained,
        peak,
        leaked,
    }
}

/// Least-squares slope of ln(y) against ln(x): the exponent k in y ~ c * x^k.
fn power_law_exponent(points: &[(f64, f64)]) -> f64 {
    let logs: Vec<(f64, f64)> = points.iter().map(|&(x, y)| (x.ln(), y.ln())).collect();
    let count = logs.len() as f64;
    let mean_x = logs.iter().map(|p| p.0).sum::<f64>() / count;
    let mean_y = logs.iter().map(|p| p.1).sum::<f64>() / count;
    let covariance: f64 = logs.iter().map(|&(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let variance: f64 = logs.iter().map(|&(x, _)| (x - mean_x).powi(2)).sum();
    covariance / variance
}

type Generator = fn(&mut Rng, usize) -> Vec<char>;

/// Input families. Unary and Fibonacci texts are the adversarial ones: they
/// make `remainder` grow large and exercise suffix links heavily.
const INPUTS: [(&str, Generator); 4] = [
    ("random, alphabet 4", |rng, n| random_text(rng, n, 4)),
    ("random, alphabet 26", |rng, n| random_text(rng, n, 26)),
    ("fibonacci word", |_, n| fibonacci_text(n)),
    ("unary (aaaa...)", |_, n| vec!['a'; n]),
];

/// Input lengths 2^min_exp, 2^(min_exp+1), ..., 2^max_exp.
fn sizes(min_exp: u32, max_exp: u32) -> Vec<usize> {
    (min_exp..=max_exp).map(|e| 1usize << e).collect()
}

fn print_table(title: &str, rows: &[Measurement]) {
    println!("\n{title}");
    println!(
        "{:>10} {:>12} {:>10} {:>14} {:>10} {:>10} {:>10}",
        "n", "time", "ns/elem", "retained B", "B/elem", "peak B/el", "leaked B"
    );
    for m in rows {
        println!(
            "{:>10} {:>12.3?} {:>10.1} {:>14} {:>10.1} {:>10.1} {:>10}",
            m.n,
            m.time,
            m.time.as_nanos() as f64 / m.n as f64,
            m.retained,
            m.retained as f64 / m.n as f64,
            m.peak as f64 / m.n as f64,
            m.leaked,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Allows for allocation granularity (collections grow in powers of two).
const MAX_MEMORY_EXPONENT: f64 = 1.10;

/// Looser than for memory: cache misses grow with the tree size.
const MAX_TIME_EXPONENT: f64 = 1.30;

#[test]
fn memory_grows_linearly() {
    let mut rng = Rng::new(2024);
    for (name, generate) in INPUTS {
        let rows: Vec<Measurement> = sizes(10, 15)
            .into_iter()
            .map(|n| measure_build(&generate(&mut rng, n)))
            .collect();
        print_table(name, &rows);

        let retained: Vec<(f64, f64)> = rows
            .iter()
            .map(|m| (m.n as f64, m.retained as f64))
            .collect();
        let peak: Vec<(f64, f64)> = rows.iter().map(|m| (m.n as f64, m.peak as f64)).collect();
        let (k_retained, k_peak) = (power_law_exponent(&retained), power_law_exponent(&peak));
        println!("memory exponent: retained {k_retained:.3}, peak {k_peak:.3}");

        assert!(
            k_retained <= MAX_MEMORY_EXPONENT,
            "{name}: retained memory grows like n^{k_retained:.3}"
        );
        assert!(
            k_peak <= MAX_MEMORY_EXPONENT,
            "{name}: peak memory grows like n^{k_peak:.3}"
        );
    }
}

#[test]
fn dropping_the_tree_frees_all_memory() {
    let mut rng = Rng::new(17);
    for (name, generate) in INPUTS {
        let m = measure_build(&generate(&mut rng, 5_000));
        assert_eq!(m.leaked, 0, "{name}: {} bytes leaked after drop", m.leaked);
    }
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "timings of unoptimised code are meaningless; run with --release"
)]
fn time_grows_linearly() {
    const REPEATS: usize = 3;
    let mut rng = Rng::new(31337);

    for (name, generate) in INPUTS {
        let mut rows = Vec::new();
        for n in sizes(13, 18) {
            let seq = generate(&mut rng, n);
            // Minimum of several runs filters out most scheduling noise.
            let best = (0..REPEATS)
                .map(|_| measure_build(&seq))
                .min_by_key(|m| m.time)
                .expect("REPEATS > 0");
            rows.push(best);
        }
        print_table(name, &rows);

        let points: Vec<(f64, f64)> = rows
            .iter()
            .map(|m| (m.n as f64, m.time.as_secs_f64()))
            .collect();
        let k = power_law_exponent(&points);
        println!("time exponent: {k:.3}");
        assert!(
            k <= MAX_TIME_EXPONENT,
            "{name}: construction time grows like n^{k:.3}"
        );
    }
}
