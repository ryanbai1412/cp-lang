// Source: CEOI 2022 P2 (Homework), https://dmoj.ca/problem/ceoi22p2
// Problem: an expression of min(A, B), max(A, B) and N <= 1e6 question marks. The marks get a
//   permutation of 1..N; print how many different values the expression can take.
// Solution: per subtree keep (lo, hi), the fewest leaves that must be smaller and larger than
//   its value: min gives (min lo, hi_A + hi_B + 1), max gives (lo_A + lo_B + 1, min hi). The
//   values lo + 1 ..= N - hi are all reachable, so print N - lo - hi. One pass over the string
//   with a stack: O(N).
// original: corpus/submissions/4727898.cpp
let s: [domain I]Char = read()

var ops: List<Int> = []          // 0 = min, 1 = max
var vals: List<(Int, Int)> = []  // (min_smaller, min_larger)
var leaves = 0
var i = 0

while i < s.len:
    let ch = s[I(i)]
    if ch == '?':
        leaves += 1
        vals.push((0, 0))
        i += 1
    elif ch == 'm':
        if s[I(i + 1)] == 'i':
            ops.push(0)
        else:
            ops.push(1)
        i += 4
    elif ch == ')':
        let op = ops.pop()!
        let rhs = vals.pop()!
        let lhs = vals.pop()!
        if op == 0:
            vals.push((min(lhs.0, rhs.0), lhs.1 + rhs.1 + 1))
        else:
            vals.push((lhs.0 + rhs.0 + 1, min(lhs.1, rhs.1)))
        i += 1
    else:
        i += 1

let root = vals[0]
print(leaves - root.0 - root.1)
