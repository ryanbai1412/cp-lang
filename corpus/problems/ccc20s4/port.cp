// Source: CCC 2020 S4 (Swapping Seats), https://dmoj.ca/problem/ccc20s4
// limits: 1000 ms, 512 MB
// Problem: N <= 1e6 people of groups A, B, C sit around a round table (a string). Print the
//   fewest swaps of two people after which each group sits in one contiguous block.
// Solution: fix where the A block starts and which group follows it: block lengths are the group
//   sizes, so every seat knows the group it wants. Everyone in a wrong seat of the first two
//   blocks must move, and one swap fixes two of them when an x sits in y's block and a y in x's.
//   Prefix counts of each letter over the doubled string give each count in O(1): O(N).
// original: corpus/submissions/4324014.cpp
let s: Str = read()
let n = s.len
domain Group = 0..3
// before[g][i]: members of g in the first i seats of the doubled table
// WISH C5: `let before = [Group] g => prefix(s.cyclic, c => c == g.letter)`
// and `before[g].sum(l..l + len)`; no `prefix` in the prelude, and nothing
// says "this range wraps around".
domain Counted = 0..2  // group 2 is what the other two leave
var before: [Counted, 0..=n]0..=1_000_000 = 0
for i in 0..n:
    for g in Counted: before[g, i + 1] = before[g, i] + Int(s[i].ord - 'A'.ord == g.ord)
fn prefix_of(g: Group, k: 0..=n) -> Int =
    if let c = Counted.at(g.ord): before[c, k] else: k - before[Counted(0), k] - before[
        Counted(1),
        k,
    ]
fn upto(
    g: Group,
    k: Int,
) -> Int = if k <= n: prefix_of(g, k) else: prefix_of(g, n) + prefix_of(g, k - n)
fn among(g: Group, from: Int, len: Int) -> Int = upto(g, from + len) - upto(g, from)
fn size(g: Group) -> Int = among(g, 0, n)

// fewest swaps when group x sits in a block followed by group y's block
fn seated(x: Group, y: Group) -> Int:
    var best = n
    for at in 0..n:
        let wrong = size(x) - among(x, at, size(x)) + size(y) - among(y, at + size(x), size(y))
        let paired = min(among(y, at, size(x)), among(x, at + size(x), size(y)))
        best min= wrong - paired
    best
print(min(seated(Group(0), Group(1)), seated(Group(0), Group(2))))
