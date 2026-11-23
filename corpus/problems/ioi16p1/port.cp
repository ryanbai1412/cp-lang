// IOI '16 P1, Detecting Molecules (multi-answer, buildable today-form)
// original: corpus/submissions/4686581.cpp (grader function find_subset)
// stdin here: n l u, then the n weights.
// stdout: the size of the subset, then its 0-based indices.
let n: 1..=200000 = read()
let l: Int = read()
let u: Int = read()
domain I = 0..n
let w: [I]Int = read()

var order: List<(Int, Int)> = []  // (weight, index), lightest first
for i in I:
    order.push((w[i], i.to_idx))
order.sort()

var lo = 0
var total = 0
var picked: List<Int> = []
for hi in 0..n:
    total += order[hi].0
    while total > u:
        total -= order[lo].0
        lo += 1
    if total >= l:
        for j in lo..(hi + 1):
            picked.push(order[j].1)
        break
print(picked.len)
print(picked)
