// CEOI 2022 P2, Homework (recursive expression-tree string I/O, buildable today-form)
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
