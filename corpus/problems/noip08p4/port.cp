// Source: NOIP 2008 Senior P4 (Two-Stack Sorting), https://www.luogu.com.cn/problem/P1155
// limits: 1000 ms, 128 MB
// Problem: a permutation of 1..n, n <= 1000, is sorted onto the output with two stacks:
//   a pushes the next input onto stack 1, b pops stack 1 to the output, c and d do the same
//   for stack 2. Print the lexicographically least op sequence (space-separated), or 0.
// Solution: positions i < j cannot share a stack when a[i] < a[j] and some later value is
//   below a[i]; the permutation is sortable iff this clash graph is bipartite. Colour each
//   component with its earliest position on stack 1, then simulate: before each push pop
//   every due value (stack 1 first), then push onto the colour's stack. O(n^2).
// original: corpus/submissions/5185830.cpp
// Bounds: n: 1..=1000; a: 1..=n
use std.graph.{two_color}

let n: 1..=1000 = read()
domain Pos = 0..n
let a: [Pos]1..=n = read()

// later[i]: the least value strictly after position i
var later: [Pos]?Int = None
for i in Pos.rev:
    if let j = i.next:
        later[i] = later[j]
        later[i] min= a[j]

// i < j clash (cannot share a stack) when a[i] < a[j] and something smaller than a[i] still
// comes after j: a[j] would sit on a[i] when a[i] must leave
fn clash(i: Pos, j: Pos) -> Bool:
    if j < i: return clash(j, i)
    if let low = later[j]: a[i] < a[j] and low < a[i] else: false
let apart = [Pos] i => [j for j in Pos if i != j and clash(i, j)]

var (stack1, stack2) = (List<Int>(), List<Int>())
var script: List<Char> = []
var need = 1
// pop every value that is due, stack 1 first: a due pop beats any push
fn pop_due():
    while true:
        if stack1.last == need:
            stack1.pop()
            script.push('b')
        elif stack2.last == need:
            stack2.pop()
            script.push('d')
        else: return
        need += 1

// the earliest position of every component colours `false`: stack 1, the cheaper letters
if let second = two_color(apart):
    for i in Pos:
        pop_due()
        if second[i]:
            stack2.push(a[i])
            script.push('c')
        else:
            stack1.push(a[i])
            script.push('a')
    pop_due()
    print(" ".join(script))
else:
    print(0)
