// COCI 2021 #6 P5 (CP Lang 0.2.2): keep one bracket per pair so kept brackets
// are balanced, greedily flipping the earliest-closing open mixed pair via `MinHeap`.
// original: corpus/submissions/4531869.cpp

@multitest
let n: Int = read()
let s: Str = read()
var pairs: List<(Int, Int)> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    pairs.push((x - 1, y - 1))

var kept: [0..2 * n]Bool = false
var opens_mixed: [0..2 * n]?Int = None
for i in 0..n:
    let (x, y) = pairs[i]
    if s[x] == s[y]:
        kept[if s[x] == '(': x else: y] = true
    else:
        opens_mixed[x] = i
        kept[if s[x] == ')': x else: y] = true

var depth = 0
var flippable = MinHeap<(Int, Int)>()
for i in 0..2 * n:
    if let p = opens_mixed[i]: flippable.push((pairs[p].1, p))
    if not kept[i]: continue
    depth += if s[i] == '(': 1 else: -1
    if depth >= 0: continue
    if let (_, p) = flippable.pop():
        let (x, y) = pairs[p]
        kept.swap(x, y)
        depth += if y <= i: 2 else: 1
    else:
        print(-1)
        return
if depth != 0:
    print(-1)
    return
print([Int(kept[y]) for (_, y) in pairs])
