// NOIP 2006 P3, Job Scheduling (buildable today-form)
// original: corpus/submissions/5165783.cpp
let m: 1..=20 = read()
let n: 1..=20 = read()
domain Machine = 1..=m
domain Job = 1..=n
domain Step = 1..=m
let order: [0..n * m]Job = read()
let machine: [Job, Step]Machine = read()
let dur: [Job, Step]1..=20 = read()

// every step is at most 20 long, so the whole schedule fits in n * m * 20
let horizon = n * m * 20
var busy: [Machine, 0..horizon]Bool = false
var ready: [Job]Int = 0
var done: [Job]Int = 0
var finish = 0
for j in order:
    done[j] += 1
    let s = Step(done[j])
    let k = machine[j, s]
    let d = dur[j, s]
    // earliest gap of length d on machine k, not before the job's last step ends
    var t = ready[j]
    while any(busy[k, u] for u in t..t + d): t += 1
    for u in t..t + d: busy[k, u] = true
    ready[j] = t + d
    finish max= ready[j]
print(finish)
