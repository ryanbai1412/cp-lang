// NCCC 8 S3: a list of n numbers holds each of 1..=k at least once. Cross
// numbers out until each of 1..=k is left exactly once, so that what is left
// is the smallest possible in dictionary order.
// original: corpus/submissions/7011313.cpp
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain I = 1..=n
domain V = 1..=k
let a: [I]V = read()

var last: [V]I = I.first  // where each number occurs for the last time
for i in I: last[a[i]] = i

// Keep a stack of the numbers chosen so far. A new number pushes out every
// larger one on top of it that occurs again later.
var kept: List<V> = []
var chosen: [V]Bool = false
for i in I:
    let x = a[i]
    if chosen[x]: continue
    while let top = kept.last and top > x and last[top] > i:
        chosen[kept.pop()!] = false
    kept.push(x)
    chosen[x] = true
print(kept)
