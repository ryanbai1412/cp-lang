// SHOI 2016 P3, Class Ranking: m students, n courses. In course i scores
// are 1..=top[i] and B is ranked rank[i] (rank - 1 students strictly
// above). Exactly k students score no higher than B in every course. The
// number of score tables, modulo 1e9+7.
// The original builds Faulhaber polynomials with a hand-written Poly type
// (70 lines). Here: the per-course count is a polynomial in top[i] of
// degree at most m, so it is brute-forced at 0..=m + 1 and interpolated.
// original: corpus/submissions/5445693.cpp
use std.mod.{Mod, Comb}

const P = 1000000007
type Z = Mod<P>

let (m, n, k): (Int, Int, Int) = read()      // m, n <= 100
domain Course = 0..n
let top: [Course]Int = read()                // up to 1e9
let rank: [Course]Int = read()
let comb = Comb<P>.new(m + 2)

// The value at x of the polynomial of degree < ys.len through (i, ys[i]). O(len).
fn lagrange_at(ys: List<Z>, x: Int) -> Z:
    let d = ys.len - 1
    if x <= d: return ys[x]
    var before = List.filled(d + 2, Z(1))    // (x - 0) * ... * (x - (i - 1))
    for i in 0..=d: before[i + 1] = before[i] * Z(x - i)
    var (after, total) = (Z(1), Z(0))        // (x - (i + 1)) * ... * (x - d)
    for i in (0..=d).rev:
        let term = ys[i] * before[i] * after * comb.inv_fact(i) * comb.inv_fact(d - i)
        total += if (d - i) % 2 == 0: term else: -term
        after *= Z(x - i)
    total

// Score tables of one course: B scores s, `below` students at most s, `above` students more.
fn tables(u: Int, above: Int, below: Int) -> Z:
    var ys = [Z(0)]
    for limit in 1..=m + 1:
        var total = Z(0)
        for s in 1..=limit: total += Z(s).pow(below) * Z(limit - s).pow(above)
        ys.push(total)
    lagrange_at(ys, u)

// still[j]: ways for the courses seen so far to leave exactly j students never above B
var still = List.filled(m, Z(0))
still[m - 1] = Z(1)
for c in Course:
    let (above, below) = (rank[c] - 1, m - rank[c])
    let ways = tables(top[c], above, below)
    // of the j candidates, `keep` stay at or below B here; the other below - keep were already out
    var next = List.filled(m, Z(0))
    for keep in 0..=below:
        for j in keep..m: next[keep] += still[j] * comb.choose(j, keep) * comb.choose(m - 1 - j, below - keep) * ways
    still = next
print(still[k])
