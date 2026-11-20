//! Running folds over a list.

/// Every prefix folded with `f`: `[a0, f(a0, a1), f(f(a0, a1), a2), ...]`, as long as `xs`.
/// `scan(xs, +)` is the running sum and `scan(xs, max)` the running maximum; no identity
/// is needed because the first value starts the fold. O(n) calls of `f`.
pub fn scan<T>(xs: List<T>, f: (T, T) -> T) -> List<T>:
    var out: List<T> = []
    for x in xs:
        out.push(if out.len == 0: x else: f(out[out.len - 1], x))
    out

/// The sums of the prefixes of `xs`: `scan(xs, +)`.
pub fn running_sum(xs: List<Int>) -> List<Int> = scan(xs, (a, b) => a + b)

@test
fn scan_folds_every_prefix():
    assert scan([3, 1, 4, 1, 5], (a, b) => a + b) == [3, 4, 8, 9, 14]
    assert scan([3, 1, 4, 1, 5], (a, b) => max(a, b)) == [3, 3, 4, 4, 5]
    assert running_sum([1, 2, 3]) == [1, 3, 6]
    let none_yet: List<Int> = []
    assert scan(none_yet, (a, b) => a + b).len == 0

@test
fn scan_agrees_with_folding_each_prefix():
    for _ in 0..100:
        let xs = [rand(-9, 9) for _ in 0..rand(1, 12)]
        let sums = running_sum(xs)
        for i in 0..xs.len: assert sums[i] == sum(xs[j] for j in 0..=i)
