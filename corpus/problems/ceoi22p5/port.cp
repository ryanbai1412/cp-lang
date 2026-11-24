// Source: CEOI 2022 P5 (Measures), https://dmoj.ca/problem/ceoi22p5
// limits: 1500 ms, 512 MB
// Problem: N people on a line, then M more added one at a time (0 <= N <= 2e5, 1 <= M <= 2e5;
//   1 <= D, positions <= 1e9). After each addition, the least time for all, moving at speed 1,
//   to stand pairwise at least D apart; print the M answers on one line, each an exact decimal
//   without trailing zeros.
// Solution: in sorted order twice the answer is max(0, max over i < j of (j - i) D - (x_j - x_i)).
//   Rank all N + M people by position offline; a SegTree over ranks keeps, for the people
//   present, the count, max(D * left - x), max(x + D * right) (left, right = how many present
//   lie left / right of a person in the node) and the best pair inside, joined by shifting with
//   counts. O((N + M) log(N + M)).
// original: corpus/submissions/4733784.cpp
let n: 0..=200_000 = read()
let m: 1..=200_000 = read()
let d: 1..=1e9 = read()
domain Person = 0..n + m
let at: [Person](1..=1e9) = read()

// The people present in a span of ranks.
struct Crowd:
    count: Int
    left: Int   // max over them of D * (how many lie left of it) - x
    right: Int  // max over them of x + D * (how many lie right of it)
    twice: Int  // twice the answer for them alone

fn join(a: Crowd, b: Crowd) -> Crowd:
    if a.count == 0: return b
    if b.count == 0: return a
    Crowd(
        count: a.count + b.count,
        left: max(a.left, b.left + d * a.count),
        right: max(b.right, a.right + d * b.count),
        twice: max(a.twice, b.twice, a.right + d + b.left),
    )

domain Rank = 0..n + m
let by_pos = Person.sort(by: p => (at[p], p))
let rank = ([Rank] r => by_pos[r.ord]).inverse()
var line = SegTree<Rank, Crowd>(Monoid(Crowd(count: 0, left: 0, right: 0, twice: 0), join))

fn half(t: Int) -> Str = if t % 2 == 1: "{t / 2}.5" else: "{t / 2}"

var answers: List<Str> = []
for p in Person:
    line.set(rank[p], Crowd(count: 1, left: -at[p], right: at[p], twice: 0))
    if p.ord >= n: answers.push(half(line.all.twice))
print(answers)
