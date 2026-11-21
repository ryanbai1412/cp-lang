// IOI 2011 Rice Hub: r rice fields lie at sorted positions on a road of
// length len. Hauling one field's rice to the hub costs the distance. With a
// budget, how many fields can one hub collect from?
// original: corpus/submissions/3732163.cpp
let r: 1..=1e5 = read()
let len: 1..=1e9 = read()
let budget: 0..=2e15 = read()
domain Field = 1..=r

fn along_the_road() -> [Field]1..=len:
    var spots: [Field]1..=len = 1
    for f in Field: spots[f] = rand(1, len)
    spots.sorted()

let x: [Field]1..=len = read(gen: along_the_road)

// The fields served are consecutive and the hub sits at their middle field.
// before[k]: total position of the first k fields, so hauling a run of
// fields costs two subtractions.
var before: [0..=r]Int = 0
for f in Field: before[f.label] = before[f.label - 1] + x[f]

fn haul(a: Int, b: Int) -> Int:            // fields a..=b, to the middle one
    let mid = (a + b) / 2
    let hub = x[Field(mid)]
    hub * (mid - a) - (before[mid - 1] - before[a - 1]) + (before[b] - before[mid]) - hub * (b - mid)

// Dropping the leftmost field never makes a run dearer, so the left end only
// moves right as the right end does.
var a = 1
var most = 0
for b in 1..=r:
    while haul(a, b) > budget: a += 1
    most max= b - a + 1
print(most)
