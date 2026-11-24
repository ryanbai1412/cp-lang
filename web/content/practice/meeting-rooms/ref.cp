let n: 1..=2e5 = read()
let talks: [0..n](s: 0..=1e9, e: 1..=1e9) = read()
var ends = Heap<Int>.new()
// Each talk frees at most one room and takes one, so `ends` never shrinks: its size is the answer.
for t in talks.sorted(by: t => t.s):
    if let first = ends.peek and first <= t.s: ends.pop()
    ends.push(t.e)
print(ends.len)
