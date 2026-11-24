// Source: CEOI 2022 P2 (Homework), https://dmoj.ca/problem/ceoi22p2
// limits: 1000 ms, 512 MB
// Problem: an expression of min(A, B), max(A, B) and N <= 1e6 question marks. The marks get a
//   permutation of 1..N; print how many different values the expression can take.
// Solution: per subtree keep (lo, hi), the fewest leaves that must be smaller and larger than
//   its value: min gives (min lo, hi_A + hi_B + 1), max gives (lo_A + lo_B + 1, min hi). The
//   values lo + 1 ..= N - hi are all reachable, so print N - lo - hi. One pass over the string
//   with a stack of pending operators ('i' of min, 'a' of max) and one of subtrees: O(N).
// original: corpus/submissions/4727898.cpp
let s: Str = read()
struct Need:
    smaller, larger: Int
var ops: List<Char> = []
var trees: List<Need> = []
for c in s:
    if c == '?': trees.push(Need(0, 0))
    elif c == 'i' or c == 'a': ops.push(c)
    elif c == ')':
        let b = trees.pop()!
        let a = trees.pop()!
        if ops.pop()! == 'i': trees.push(Need(min(a.smaller, b.smaller), a.larger + b.larger + 1))
        else: trees.push(Need(a.smaller + b.smaller + 1, min(a.larger, b.larger)))
let root = trees.last!
print(s.count(c => c == '?') - root.smaller - root.larger)
