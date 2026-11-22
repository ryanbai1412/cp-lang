// cpl 0.3.0
// Problem: range add, range minimum.
// Input: n q, a_1..a_n, then q queries: "1 l r x" adds x to a_l..a_r,
// "2 l r" asks min(a_l..a_r). 1-based, |a_i|, |x| <= 1e9.
// Guesses: H-ENUM-TAG-INT (`@tag(1)` reads the integer token 1),
// H-OP-NAMES (`add` as the tag op next to `min` as the fold op),
// H-NODE-WIDTH-ACT (nodes widen to Int under `act:`).

let n = read<1..=2e5>()
let q = read<1..=2e5>()
domain V = 1..=n
let a = read<[V]-1e9..=1e9>()
var st = SegTree(min, a, act: add)

enum Query:
    @tag(1) Add(l: V, r: V, x: -1e9..=1e9)
    @tag(2) Ask(l: V, r: V)

for _ in 0..q:
    match read<Query>():
        Add(l, r, x) => st.apply(l..=r, x)
        Ask(l, r) => print(st.fold(l..=r))
