// NOIP 2006 P3, Job Scheduling
// original: corpus/submissions/5165783.cpp
let m: 1..=20 = read()
let n: 1..=20 = read()
domain Machine = 1..=m
domain Job = 1..=n
domain Step = 1..=m
let order: [n * m]Job = read()
let machine: [Job, Step]Machine = read()
let dur: [Job, Step]1..=20 = read()

// every step is at most 20 long, so the whole schedule fits in n * m * 20
var busy: [Machine, 0..n * m * 20]Bool = false
var ready: [Job]Int = 0
var next: [Job]Step = Step.first
for j in order:
    let s = next[j]
    if let after = s + 1: next[j] = after
    let (k, d) = (machine[j, s], dur[j, s])
    // earliest gap of length d on machine k, not before the job's last step ends
    var t = ready[j]
    while any(busy[k, u] for u in t..t + d): t += 1
    for u in t..t + d: busy[k, u] = true
    ready[j] = t + d
print(max(ready)!)
