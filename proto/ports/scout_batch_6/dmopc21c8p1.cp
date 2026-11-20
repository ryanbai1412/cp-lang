// DMOPC '21 Contest 8 P1 Peanut Planning: put n numbers in a line so that
// every two neighbours add up to at least m, and the line is the smallest
// possible in dictionary order. Print it, or -1 if there is none.
// original: corpus/submissions/4522590.cpp
let (n, m): (1..=1e6, 1..=2e9) = read()                                             // WISH S8
let a: [n]1..=1e9 = read()

// The line takes the smallest number left, then the smallest that is a good
// neighbour for it, and so on. The smallest numbers only grow, so a number
// that was once a good neighbour stays one: `small` gives numbers out from its
// front and promotes them from its back, and the promoted ones form a stack
// with the smallest on top.
var small = Deque(sorted(a))                                                        // WISH S12
var partners: List<Int> = []
var line: List<Int> = []
while let x = small.pop_front() ?? partners.pop():                                  // WISH S12
    line.push(x)
    for big in small.pop_back_while(b => b + x >= m): partners.push(big)            // WISH S12
    if line.len == n: break
    let y = partners.pop() else:
        print(-1)
        return                                                                      // WISH S7
    line.push(y)
print(line)
