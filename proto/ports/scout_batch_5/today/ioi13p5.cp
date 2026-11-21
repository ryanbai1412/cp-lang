// IOI 2013 Robots, as it builds today: sizes are negated for the min-heap and
// the cursor over the sorted toys is an index (K9, K10, K6).
// original: corpus/submissions/4688418.cpp (a grader-style `putaway`; check.sh adds a main)
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
            if left.pop() == none: break
    while next < t:
        left.push(-by_weight[next].size)
        next += 1
    for limit in small_up.rev:
        for _ in 0..minutes:
            if let s = left.pop():
                if -s >= limit: return false
    left.len() == 0

domain Minutes = 1..=t
if let m = Minutes.first_where(m => done_in(m.label)): print(m)
else: print(-1)
