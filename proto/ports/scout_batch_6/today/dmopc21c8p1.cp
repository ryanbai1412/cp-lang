// DMOPC '21 Contest 8 P1 Peanut Planning: put n numbers in a line so that
// every two neighbours add up to at least m, and the line is the smallest
// possible in dictionary order. Print it, or -1 if there is none.
// original: corpus/submissions/4522590.cpp
let n: 1..=1e6 = read()
let m: 1..=2e9 = read()
let a: [n]1..=1e9 = read()

// The line takes the smallest number left, then the smallest that is a good
// neighbour for it, and so on. The smallest numbers only grow, so a number
// that was once a good neighbour stays one: `small` gives numbers out from its
// front and promotes them from its back, and the promoted ones form a stack
// with the smallest on top.
var small = sorted([x for x in a])
var front = 0
var partners: List<Int> = []
var line: List<Int> = []
var stuck = false
while line.len < n and not stuck:
    var x = 0
    if front < small.len:
        x = small[front]
        front += 1
    else: x = partners.pop()!
    line.push(x)
    while front < small.len and small.last! + x >= m: partners.push(small.pop()!)
    if line.len < n:
        if let y = partners.pop(): line.push(y)
        else: stuck = true
if stuck: print(-1)
else: print(line)
