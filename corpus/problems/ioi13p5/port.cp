// Source: IOI 2013, Robots, https://dmoj.ca/problem/ioi13p5
// Problem: T <= 1e6 toys with weight and size; A weak robots carry toys of weight strictly below
//   their limit, B small robots toys of size strictly below theirs (A, B <= 5e4, values <= 2e9).
//   Each robot puts away one toy per minute. Print the fewest minutes for all toys, or -1.
// Solution: binary search the minutes m. Check: weak robots by rising limit each take the m
//   largest-size toys among those light enough (a heap of sizes, toys sorted by weight); then
//   small robots by falling limit each take the m largest left, failing when one is too big.
//   O(T log^2 T).
// As it builds today: sizes are negated for the min-heap and the cursor over the sorted toys is
//   an index (K9, K10, K6).
// original: corpus/submissions/4688418.cpp (a grader-style `putaway`; check.sh adds a main)
// Bounds: a, b: 0..=5e4; t: 1..=1e6; weak, small, toys.weight, toys.size: 1..=2e9
let a: 0..=5e4 = read()
let b: 0..=5e4 = read()
let t: 1..=1e6 = read()
let weak: [a]1..=2e9 = read()
let small: [b]1..=2e9 = read()
let toys: [t](weight: 1..=2e9, size: 1..=2e9) = read()

let by_weight = toys.sorted()
let weak_up = weak.sorted()
let small_up = small.sorted()

fn done_in(minutes: Int) -> Bool:
    var left = MinHeap<Int>()
    var next = 0
    for limit in weak_up:
        while next < t and by_weight[next].weight < limit:
            left.push(-by_weight[next].size)
            next += 1
        for _ in 0..minutes:
            if left.pop() == None: break
    while next < t:
        left.push(-by_weight[next].size)
        next += 1
    for limit in small_up.rev:
        for _ in 0..minutes:
            if let s = left.pop():
                if -s >= limit: return false
    left.len == 0

domain Minutes = 1..=t
if let m = Minutes.first_true(m => done_in(m.label)): print(m)
else: print(-1)
