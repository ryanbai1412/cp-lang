// Source: CCC 2023 S5 (The Filter), https://dmoj.ca/problem/ccc23s5
// Problem: 2 <= n <= 1e9. Print, in increasing order, every x in 0..=n with x / n in the
//   Cantor set; there are at most 1e6 of them.
// Solution: walk the Cantor construction: a segment [k / b, (k + 1) / b] holding numerators
//   l..=r keeps its left and right thirds, so about n^log3(2) segments are visited. Once a
//   segment holds at most 11 numerators, test each by long division of x / n in base 3
//   until a remainder repeats (Floyd's cycle walk, no set); a digit 1 is fine only when
//   nothing follows it.
// original: corpus/submissions/5350810.cpp
let n: 2..=1e9 = read()

fn in_cantor(x: 0..=n) -> Bool:
    // the remainders of the long division of x / n in base 3 repeat eventually (Floyd's
    // tortoise and hare: the hare passes every remainder before they meet); a digit 1 is
    // fine only when nothing follows it
    fn step(rem: Int) -> ?Int:
        let next = rem % n * 3
        if rem / n == 1 and next != 0: None else: Some(next)
    var slow = x
    var fast = x
    while true:
        repeat 2:
            fast = step(fast) ?? return false
        slow = step(slow)!
        if slow == fast: return true

// the numerators l..=r of the segment [k / b, (k + 1) / b] that survive
var kept: List<0..=n> = []
fn keep(l: 0..=n, r: 0..=n, k: Int, b: Int):
    if r - l <= 10:
        for x in l..=r:
            if in_cantor(x): kept.push(x)
        return
    keep(l, n * (3 * k + 1) / (3 * b), 3 * k, 3 * b)
    keep((n * (3 * k + 2) + 3 * b - 1) / (3 * b), r, 3 * k + 2, 3 * b)
keep(0, n, 0, 1)
for x in kept: print(x)
