// NOIP 2008 P4, twin-stack sorting: the lexicographically least a/b/c/d script that sorts a permutation
// original: corpus/submissions/5185830.cpp
let n: 1..=1000 = read()
domain P = 0..n
let a: [P]Int = read()

// i < j clash (cannot share a stack) when a[i] < a[j] and something smaller
// than a[i] still comes after j: a[j] would sit on a[i] when a[i] must leave
let later: [P]Int = [P] j => min(a[k] for k in j.r..) ?? n + 1     // least value after j
let clashes = [(i, j) for i in P for j in i.r.. if a[i] < a[j] and later[j] < a[i]]

// the earliest position of every component goes to stack 0, the cheaper letters
let side = two_color(Graph.undirected(P, clashes)) ?? return print(0)

var stack: [Bool]List<Int> = []
var script: List<Char> = []
var (next, need): (?P, Int) = (P.first, 1)
while need <= n:
    // a pop that is due beats everything on its own stack; between the
    // stacks the letters decide: a < b < c < d
    let push = if let i = next and (stack[side[i]].last ?? n + 1) > a[i]: side[i] else: none
    if stack[false].last == need:
        script.push('b')
        stack[false].pop()
        need += 1
    elif push == false:
        script.push('a')
        stack[false].push(a[next!])
        next = next! + 1
    elif stack[true].last == need:
        script.push('d')
        stack[true].pop()
        need += 1
    else:
        script.push('c')
        stack[true].push(a[next!])
        next = next! + 1
print(script)
