// Source: IOI 2021 Day 2 P1 (Mutating DNA), https://dmoj.ca/problem/ioi21p4
// Problem: strings a, b of n <= 1e5 letters from A, C, T and q <= 1e5 queries x <= y (0-based):
//   print the fewest swaps of two letters of a[x..=y] that turn it into b[x..=y], or -1.
// Solution: only the places where they differ matter, and of those only how many have each
//   (letter of a, letter of b) pair: prefix counts of the nine pairs. Impossible when some
//   letter is needed more often than it is there. Otherwise a place wanting X where another
//   wants its letter back is fixed with one swap, and what is left over goes round in threes,
//   two swaps each. O(n + q).
// original: corpus/submissions/4698075.cpp
domain Base = 0..3
fn base(c: Char) -> Base = Base(if c == 'A': 0 elif c == 'C': 1 else: 2)

let (n, q): (Int, Int) = read()
let (a, b): (Str, Str) = read()
// upto[i][x, y]: the places before i where a has x and b has y.
var upto: List<[Base, Base]Int> = [[Base, Base] (x, y) => 0]
for i in 0..n:
    var next = upto.last!.clone()
    next[base(a[i]), base(b[i])] += 1
    upto.push(next)

for _ in 0..q:
    let (x, y): (Int, Int) = read()
    let have = [Base, Base] (s, t) => upto[y + 1][s, t] - upto[x][s, t]
    var possible = true
    for s in Base:
        if sum(have[s, t] for t in Base) != sum(have[t, s] for t in Base): possible = false
    if not possible:
        print(-1)
        continue
    var (swaps, left) = (0, 0)
    for s in Base:
        for t in Base:
            if s < t:
                let pairs = min(have[s, t], have[t, s])
                swaps += pairs
                left += have[s, t] + have[t, s] - 2 * pairs
    print(swaps + left / 3 * 2)
