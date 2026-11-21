//! Worked contest examples: write the obvious brute force, dump its small answers into
//! the `spec` helpers, read off the structure, and only then write the fast solution.
//! Each test is one problem, start to finish.
use std.spec.{poly_degree}
use lib.spec.recurrence.{integer_recurrence, show_recurrence, nth_term, extend}
use lib.spec.interp.{closed_form, show_poly, lagrange_at}
use lib.spec.structure.{split_report, split_dp, interval_report, period, periodic_at}

/// Domino tilings of a 3 x n board, by a broken-profile DP: `mask` marks which of the
/// next three cells (in column order) are already covered.
fn tilings(n: Int) -> Int:
    var ways = List.filled(8, 0)
    ways[0] = 1
    for cell in 0..3 * n:
        var next = List.filled(8, 0)
        for mask in 0..8:
            if ways[mask] == 0: continue
            if mask % 2 == 1:
                next[mask / 2] += ways[mask]
                continue
            // a vertical domino covers the cell below, a horizontal one the cell three ahead
            if cell % 3 < 2 and mask / 2 % 2 == 0: next[(mask + 2) / 2] += ways[mask]
            if cell + 3 < 3 * n: next[(mask + 8) / 2] += ways[mask]
        ways = next
    ways[0]

@test
fn problem_count_tilings_of_a_3_by_n_board_for_n_up_to_1e18():
    // 1. the brute force answers n = 0..12
    let small = [tilings(n) for n in 0..12]
    assert small == [1, 0, 3, 0, 11, 0, 41, 0, 153, 0, 571, 0]
    // 2. is it a linear recurrence? yes, and it prints the way one would write it
    let c = integer_recurrence(small)!
    assert show_recurrence(c) == "a[n] = 4 a[n-2] - a[n-4]"
    // 3. it predicts a value the search never saw
    assert extend(c, small, 15)[14] == tilings(14)
    // 4. the answer for the real constraints, modulo 10^9 + 7
    let big = nth_term::<1000000007>(c, small, 10 ** 18)
    assert big == nth_term::<1000000007>([4, -1], [1, 3], 5 * 10 ** 17)

@test
fn problem_count_sorted_triples_up_to_n():
    // how many (a, b, c) with 1 <= a <= b <= c <= n? brute force for n = 0..7
    let brute = (n) => sum(1 for a in 1..=n for b in a..=n for c in b..=n)
    let small = [brute(n) for n in 0..8]
    // 1. the differences die out: a cubic
    assert poly_degree(small) == 3
    // 2. its exact closed form
    assert show_poly(closed_form(small)!) == "1/6 n^3 + 1/2 n^2 + 1/3 n"
    // 3. or skip the formula and evaluate straight from the samples, modulo a prime
    type M = Mod<998244353>
    let n = 10 ** 12
    assert lagrange_at([M(x) for x in small], n) == M(n) * M(n + 1) * M(n + 2) / M(6)

@test
fn problem_split_an_array_into_k_groups_minimizing_squared_sums():
    // the O(k n^2) DP is too slow for n = 10^5; which optimization fits?
    let a = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3]
    var pre = [0]
    for x in a: pre.push(pre.last + x)
    let cost = (i, j) => (pre[j] - pre[i]) ** 2
    assert split_report(cost, a.len, 4) == [
        "quadrangle inequality holds: cost is Monge",
        "split points are monotone: divide and conquer, O(k n log n)",
        "cost is convex in the number of groups: Aliens trick",
    ]
    // the same table the report was read from
    let (dp, opt) = split_dp(cost, a.len, 4)
    assert dp[1][a.len] == 39 ** 2 and opt[2][a.len] == 6

@test
fn problem_merge_adjacent_piles_at_minimum_cost():
    // interval DP, O(n^3); n = 5000 needs Knuth. Is it allowed?
    let piles = [4, 1, 1, 4, 7, 2, 2, 3]
    var pre = [0]
    for x in piles: pre.push(pre.last + x)
    let report = interval_report((i, j) => pre[j] - pre[i], piles.len)
    assert report.last == "split points are monotone, as the theory promises: Knuth, O(n^2)"
    // a cost with a bonus for long merges only looks fine on lucky inputs
    let risky = interval_report((i, j) => pre[j] - pre[i] - (j - i) ** 2, piles.len)
    assert not risky.contains("quadrangle inequality holds")

@test
fn problem_who_wins_taking_1_3_or_4_stones_from_a_pile_of_1e18():
    // Grundy values by definition for small piles
    var g = List<Int>()
    for n in 0..40:
        let seen = [g[n - t] for t in [1, 3, 4] if t <= n]
        var mex = 0
        while seen.contains(mex): mex += 1
        g.push(mex)
    // they repeat every 7 from the start, so the table for 10^18 is seven numbers
    assert period(g) == (0, 7)
    assert g[0..7] == [0, 1, 0, 1, 2, 3, 2]
    assert periodic_at(g, 10 ** 18) == g[10 ** 18 % 7]
    // with a preperiod: a rule that only applies to big piles
    var h = List<Int>()
    for n in 0..60:
        let seen = [h[n - t] for t in [2, 5] if t <= n] + (if n >= 9: [h[0]] else: [])
        var mex = 0
        while seen.contains(mex): mex += 1
        h.push(mex)
    let (start, p) = period(h)!
    for n in start..h.len - p: assert h[n] == h[n + p]
    assert periodic_at(h, 10 ** 18) == h[start + (10 ** 18 - start) % p]
