// Source: CCC 2023 S5 (The Filter), https://dmoj.ca/problem/ccc23s5
// Problem: 2 <= n <= 1e9. Print, in increasing order, every x in 0..=n with x / n in the
//   Cantor set; there are at most 1e6 of them.
// Solution: walk the Cantor construction: a segment [k / b, (k + 1) / b] holding numerators
//   l..=r keeps its left and right thirds, so about n^log3(2) segments are visited. Once a
//   segment holds at most 11 numerators, test each by long division of x / n in base 3
//   until a remainder repeats; a digit 1 is fine only when nothing follows it.
let n: 2..=1e9 = read()

fn in_cantor(x: Int) -> Bool:
    // ternary digits of x / n by long division; a digit 1 is fine only when nothing follows it
    var seen: Set<Int> = []
    var rem = x
    while true:
        let digit = rem / n
        rem = rem % n * 3
        if digit == 1 and rem != 0: return false
        if rem in seen: return true
        seen.insert(rem)

var kept: List<Int> = []
fn filter(l: Int, r: Int, k: Int, b: Int):
    if r - l <= 10:
        for x in l..=r:
            if in_cantor(x): kept.push(x)
        return
    filter(l, n * (3 * k + 1) / (3 * b), 3 * k, 3 * b)
    filter((n * (3 * k + 2) + 3 * b - 1) / (3 * b), r, 3 * k + 2, 3 * b)
filter(0, n, 0, 1)
for x in kept: print(x)
