// NOIP 2008 P4, twin-stack sorting: the lexicographically least a/b/c/d script that sorts a permutation (buildable today-form)
// original: corpus/submissions/5185830.cpp
// The clash graph is an adjacency list coloured by a hand-rolled walk, and the
// script is built as a List<Char> printed with write: the prototype has no
// two_color and no join.
let n: 1..=1000 = read()
let a: [0..n]Int = read()

// later[i]: the least value strictly after position i
var later: [0..n]Int = n + 1
for i in (0..n - 1).rev: later[i] = min(later[i + 1], a[i + 1])

// i < j clash (cannot share a stack) when a[i] < a[j] and something smaller
// than a[i] still comes after j: a[j] would sit on a[i] when a[i] must leave
var clash: [0..n]List<Int> = []
for i in 0..n:
    for j in i + 1..n:
        if a[i] < a[j] and later[j] < a[i]:
            clash[i].push(j)
            clash[j].push(i)

// the earliest position of every component goes to stack 0, the cheaper letters
var side: [0..n]?Int = None
var ok = true
for s in 0..n:
    if side[s] != none: continue
    side[s] = 0
    var todo: List<Int> = [s]
    while let i = todo.pop():
        for j in clash[i]:
            if side[j] == None:
                side[j] = 1 - side[i]!
                todo.push(j)
            elif side[j] == side[i]: ok = false

// a stack only ever takes a value smaller than its top
fn fits(stack: List<Int>, x: Int) -> Bool:
    if let top = stack.last: return x < top
    true

if not ok: print(0)
else:
    var stack0: List<Int> = []
    var stack1: List<Int> = []
    var script: List<Char> = []
    var (next, need) = (0, 1)
    while need <= n:
        // a pop that is due beats everything on its own stack; between the
        // stacks the letters decide: a < b < c < d
        if stack0.last == need:
            stack0.pop()
            script.push('b')
            need += 1
        elif next < n and side[next] == 0 and fits(stack0, a[next]):
            stack0.push(a[next])
            script.push('a')
            next += 1
        elif stack1.last == need:
            stack1.pop()
            script.push('d')
            need += 1
        else:
            stack1.push(a[next])
            script.push('c')
            next += 1
    for i in 0..script.len:
        if i > 0: print(" ", end: "")
        print(script[i], end: "")
    print("")
