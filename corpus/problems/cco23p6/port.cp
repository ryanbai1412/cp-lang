// Source: CCO 2023 Day 2 P3 (Triangle Collection), https://dmoj.ca/problem/cco23p6
// Problem: c_l <= 1e9 sticks of each length l <= N <= 2e5. An isosceles triangle takes two sticks
//   of length l and a third of length 1..=2l-1; each stick is used once. After each of Q <= 2e5
//   events (count of length l changes by d), print the most triangles.
// Solution: split each length into c/2 pairs and c%2 singles. First match singles as third sides:
//   a pair of length i takes a single of length <= 2i-1, so a SegTree over lengths keeps (made,
//   free singles, open pairs), with pairs entered at 2i-1 and the left part's singles matched to
//   the right part's pairs. The unmatched pairs make 2 triangles per 3 pairs: made +
//   (pairs - made) * 2 / 3. An event updates two leaves: O(log N).
// As it builds today: the monoid is a free function behind a lambda, and the
//   two derived tables are refreshed by hand (K13, F7).
// original: corpus/submissions/5618725.cpp
let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain Len = 1..=n
var have: [Len]0..=1e9 = read()
let changes: [q](len: Len, by: -1e9..=1e9) = read()

struct Run:
    made, singles, pairs: Int
fn join(a: Run, b: Run) -> Run:
    let k = min(a.singles, b.pairs)
    Run(a.made + b.made + k, a.singles + b.singles - k, a.pairs + b.pairs - k)

fn reach(i: Len) -> Len: Len(min(n, 2 * i.label - 1))
var ending: [Len]Int = 0
var pairs = 0
var runs = SegTree<Len, Run>(Monoid(Run(0, 0, 0), join))
fn refresh(i: Len):
    runs.set(i, join(Run(0, have[i] % 2, 0), Run(0, 0, ending[i])))
fn count(i: Len, sign: Int):
    pairs += sign * (have[i] / 2)
    ending[reach(i)] += sign * (have[i] / 2)
    refresh(i)
    refresh(reach(i))
for i in Len: count(i, 1)
for (i, by) in changes:
    count(i, -1)
    have[i] += by
    count(i, 1)
    let made = runs.fold(Len.all).made
    print(made + (pairs - made) * 2 / 3)
