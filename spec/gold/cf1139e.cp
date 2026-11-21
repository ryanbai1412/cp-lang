// Each day a student leaves; then pick ≤ 1 student per club to maximise the mex of potentials.
// mex > v iff potentials 0..=v match distinct clubs. Backwards in time students only arrive,
// so keep one matching and push the mex up while it augments.
use std.flow.{Matching}
let n: 1..=5000 = read()
let m: 1..=n = read()
domain Student = 1..=n
domain Club = 1..=m
domain Potential = 0..5000
let p: [Student]Potential = read()
let c: [Student]Club = read()
let d: 1..=n = read()
let leaves = read<[d]Student>()

var stays: [Student]Bool = true
for s in leaves: stays[s] = false

var matching = Matching<Potential, Club>()
for s in Student if stays[s]: matching.add(p[s], c[s])

var mex: Potential.bounds = 0   // every potential before this cut is matched
var answer: [d]Potential.bounds = 0
for (day, s) in leaves.items.rev:
    while let v = mex.next and matching.augment(from: v): mex = v.after
    answer[day] = mex
    matching.add(p[s], c[s])
for x in answer: print(x)
