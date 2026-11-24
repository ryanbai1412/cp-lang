// Source: Codeforces 2002E (Cosmic Rays), https://codeforces.com/contest/2002/problem/E
// limits: 2000 ms, 500 MB
// Problem: a sequence given as n pairs (a_i, b_i), a_i <= 1e9 copies of b_i, adjacent b's
//   distinct. Each second the first element and every element unequal to its predecessor
//   vanish. For each prefix of pairs print its strength, the seconds until it is empty.
//   Multitest, sum of n <= 3e5.
// Solution: a run of length a alone dies at time a; if the runs between it and an earlier run
//   of the same value all die first, at time t, the two merge and die at a + a' - t. A stack
//   keeps runs with strictly falling death times: a new run pops every run it outlives,
//   noting when the last popped one died, and merges with a same-value run it reaches. The
//   strength is the bottom run's death time. Amortized O(n).
@!multitest
let n: 1..=300_000 = read()
var stack: List<(dies: Int, val: Int)> = []
var strength: List<Int> = []
for _ in 0..n:
    var dies: Int = read<1..=1_000_000_000>()
    let val: 0..=n = read()
    var gone = 0  // when the runs popped so far died
    while let top = stack.last:
        if top.val != val and dies < top.dies: break
        if top.val == val: dies += top.dies - gone
        gone = top.dies
        stack.pop()
    stack.push((dies:, val:))
    strength.push(stack[0].dies)
print(strength)
