//! Sums of floors of a linear function, in O(log m): counting lattice points under a
//! line, and summing residues of an arithmetic progression.

/// `sum(floor((a * i + b) / m) for i in 0..n)`, for any signed `a` and `b`, `m > 0`
/// and `n >= 0`. O(log m), the same steps as Euclid's algorithm.
///
/// The answer and `a * n + b` (after `a` and `b` are reduced modulo `m`) must fit an `Int`.
pub fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int:
    assert n >= 0 and m > 0
    var (count, den, slope, start) = (n, m, a, b)
    var total = 0
    while true:
        // take out the whole multiples of `den`; `/` and `%` round down, so signs are fine
        total += count * (count - 1) / 2 * (slope / den) + count * (start / den)
        slope %= den
        start %= den
        let top = slope * count + start
        if top < den: break
        // count the lattice points under the line by columns instead of rows
        count = top / den
        start = top % den
        swap(mut den, mut slope)
    total

/// `sum((a * i + b) % m for i in 0..n)`: the residues of an arithmetic progression.
pub fn mod_sum(n: Int, m: Int, a: Int, b: Int) -> Int:
    a * (n * (n - 1) / 2) + b * n - m * floor_sum(n, m, a, b)

/// How many `i in 0..n` have `(a * i + b) % m` in `lo..hi`, for `0 <= lo <= hi <= m`.
pub fn count_residues(n: Int, m: Int, a: Int, b: Int, lo: Int, hi: Int) -> Int:
    assert 0 <= lo <= hi <= m
    // floor((x - lo) / m) - floor((x - hi) / m) is 1 exactly when x % m is in lo..hi
    floor_sum(n, m, a, b - lo) - floor_sum(n, m, a, b - hi)

@test
fn agrees_with_the_loop_for_every_sign():
    for _ in 0..400:
        let (n, m, a, b) = (rand(0, 30), rand(1, 20), rand(-40, 40), rand(-40, 40))
        assert floor_sum(n, m, a, b) == sum((a * i + b) / m for i in 0..n)
        assert mod_sum(n, m, a, b) == sum((a * i + b) % m for i in 0..n)
        let lo = rand(0, m)
        let hi = rand(lo, m)
        assert count_residues(n, m, a, b, lo, hi) == count(lo <= (a * i + b) % m < hi for i in 0..n)

@test
fn large_arguments_stay_exact():
    assert floor_sum(1_000_000_000, 1, 1, 0) == 499_999_999_500_000_000
    assert floor_sum(1_000_000_000, 1_000_000_007, 998_244_353, 123_456_789) == 499122172130479402
    assert floor_sum(0, 5, 3, 2) == 0 and floor_sum(4, 7, 0, -1) == -4

@test
fn lattice_points_in_a_triangle():
    // points with x >= 1, y >= 1 and 3x + 5y <= 100: for each x, floor((100 - 3x) / 5) of them
    let inside = count(3 * x + 5 * y <= 100 for x in 1..=33 for y in 1..=20)
    assert floor_sum(33, 5, -3, 97) == inside
