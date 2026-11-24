// Source: IOI 2021 Day 2 P1 (Mutating DNA), https://dmoj.ca/problem/ioi21p4
// Problem: strings a, b of n <= 1e5 letters from A, C, T and q <= 1e5 queries x <= y (0-based):
//   print the fewest swaps of two letters of a[x..=y] that turn it into b[x..=y], or -1.
// Solution: only the places where they differ matter, and of those only how many have each
//   (letter of a, letter of b) pair: prefix counts of the nine pairs. Impossible when some
//   letter is needed more often than it is there. Otherwise a place wanting X where another
//   wants its letter back is fixed with one swap, and what is left over goes round in threes,
//   two swaps each. O(n + q).
// original: corpus/submissions/4698075.cpp (grader functions init / get_distance)
// stdin: n q, a, b, then q queries x y. stdout: the answer per query.
// Bounds: n, q: 1..=100_000; x, y: 0..n
let (n, q): (1..=100_000, 1..=100_000) = read()
domain Pos = 0..n
domain Base = 0..3
fn base(c: Char) -> Base = Base(if c == 'A': 0 elif c == 'C': 1 else: 2)
let (a, b): (Str, Str) = read()
assert a.len == n and b.len == n

// count[s, t]: prefix counts of the places where a has s and b has t
let at = [Pos] i => (base(a[Int(i)]), base(b[Int(i)]))
let count: [Base, Base]Prefix<Pos, Int> = [Base, Base] (s, t) =>
    Prefix<Pos, Int>([Pos] i => if at[i] == (s, t): 1 else: 0)

for _ in 0..q:
    let (x, y): (Pos, Pos) = read()
    assert x <= y
    let have = [Base, Base] (s, t) => count[s, t].fold(x..=y)
    if any(sum(have[s, t] for t in Base) != sum(have[t, s] for t in Base) for s in Base):
        print(-1)
    else:
        // a place wanting t where another wants its s back is one swap; the rest go round in
        // threes, two swaps each
        let swaps = sum(min(have[p.lo, p.hi], have[p.hi, p.lo]) for p in Pair<Base>)
        let wrong = sum(have[s, t] for s in Base for t in Base if s != t)
        print(swaps + (wrong - 2 * swaps) / 3 * 2)
