// cp: 0.1 456e4a4+dirty
// NOI 2016 Intervals: pick m of the n intervals that share a point, with the
// longest and the shortest picked as close in length as possible.
// Sorted by length, the picks may as well be a window: slide it, keeping how
// deeply every coordinate is covered.
let n: 1..=500000 = read()
let m: 1..=200000 = read()
type Span = (l: 0..=1e9, r: 0..=1e9)
domain I = 0..n
fn short() -> Span:                         // few coordinates, so intervals pile up
    let l = rand(0, 8)
    (l, l + rand(0, 5))
fn tangle() -> [I]Span: [I] _ => short()
let given: [I]Span = read(gen: tangle) where all(s.l <= s.r for s in given)
fn length(s: Span) -> Int: s.r - s.l
let spans = given.sorted(by: s => length(s))

var ends: List<Int> = []
for s in spans:
    ends.push(s.l)
    ends.push(s.r)
let (X, coord) = compress(ends)
var depth = AddMax<X>()
fn cover(s: Span, times: Int):
    depth.add(coord.first_ge(s.l)!, coord.first_ge(s.r)!, times)

var best: ?Int = none
var taken = 0                               // the window is spans[i..taken]
for i in 0..n:
    while taken < n and depth.max() < m:
        cover(spans[taken], 1)
        taken += 1
    if depth.max() >= m:
        best min= length(spans[taken - 1]) - length(spans[i])
    cover(spans[i], -1)
print(best ?? -1)
