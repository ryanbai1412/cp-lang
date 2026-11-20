// IOI 2011 Rice Hub: rice fields stand along a road. Carrying one field's
// rice to the hub costs the distance, and the budget is b. How many fields
// can one hub serve?
// original: corpus/submissions/3732163.cpp
let n: 1..=1e5 = read()
let len: 1..=1e9 = read()
let budget: 0..=2e15 = read()
domain Field = 1..=n
let x: Sorted<[Field]1..=len> = read()                              // WISH R5
let pre = prefix(x)

// the hub goes on the middle field of the block
fn cost(w: Range1<Field>) -> Int:
    let hub = w.mid                                                 // WISH R1
    (pre.sum(hub..=w.last) - pre.sum(w.first..=hub)) - x[hub] * ((w.last - hub) - (hub - w.first))

// a block inside an affordable block is affordable, so the left end only moves right
print(max(w.len for w in sliding(Field, ok: w => cost(w) <= budget)))     // WISH R3
