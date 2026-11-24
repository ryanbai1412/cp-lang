// Source: NOIP 2006 Senior P3 (Job Scheduling Plan), Luogu P1065 (not on DMOJ)
// Problem: n < 20 jobs of m < 20 steps each; step k of job j runs on a given machine for a given
//   time <= 20. Steps are scheduled one by one in a given order, each at the earliest time that
//   follows the job's previous step and fits a free gap on its machine. Print the makespan.
// Solution: simulate with a busy flag per machine and time unit; the schedule ends by n * m * 20.
//   For each step scan t upward from the job's ready time until [t, t + d) is free on its
//   machine, then mark it. O(n m * horizon * 20) worst case, tiny here.
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
