// Source: CCC 2020 S4 (Swapping Seats), https://dmoj.ca/problem/ccc20s4
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
var before: [Group]List<Int> = [Group] _ => [0]
for i in 0..2 * n:
    for g in Group:
        before[g].push(before[g][i] + Int(s[i % n].ord - 'A'.ord == g.ord))
fn among(g: Group, from: Int, len: Int) -> Int = before[g][from + len] - before[g][from]
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
