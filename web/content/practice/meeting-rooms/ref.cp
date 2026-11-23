let n: 1..=2e5 = read()
let talks: [0..n](s: 0..=1e9, e: 1..=1e9) = read()
var ends = Heap<Int>.new()
var rooms = 0
for t in talks.sorted(by: t => t.s):
    if let first = ends.peek and first <= t.s: ends.pop()
    ends.push(t.e)
    rooms max= ends.len
print(rooms)
