// IOI 2015 Day 2 P2, Sorting (buildable today-form)
// original: corpus/submissions/5723358.cpp (grader function findSwapPairs)
// stdin: n, the permutation, m, then Ermek's m swaps. stdout: the number of
// rounds r, then our r swaps.
let n: 1..=200000 = read()
let start: [0..n]Int = read()
let m: 1..=600000 = read()
let ermek: [0..2 * m]Int = read()

var val: List<Int> = []              // val[slot]: scratch copy of the row
var slot_of: List<Int> = []          // inverse of val
for i in 0..n:
    val.push(0)
    slot_of.push(0)
fn reset():
    for i in 0..n:
        val[i] = start[i]
        slot_of[start[i]] = i
fn swap_slots(a: Int, b: Int):
    let (x, y) = (val[a], val[b])
    val[a] = y
    val[b] = x
    slot_of[y] = a
    slot_of[x] = b

// After Ermek's first t swaps the row needs n - (number of cycles) swaps.
// Our swaps can be chosen on values instead of slots, so they commute with
// his: t rounds are enough exactly when that count is at most t.
fn swaps_after(t: Int) -> Int:
    reset()
    for r in 0..t: swap_slots(ermek[2 * r], ermek[2 * r + 1])
    var seen: [0..n]Bool = false
    var cycles = 0
    for i in 0..n:
        if not seen[i]:
            cycles += 1
            var j = i
            while not seen[j]:
                seen[j] = true
                j = val[j]
    n - cycles

var lo = 0
var hi = m
while lo < hi:
    let mid = (lo + hi) / 2
    if swaps_after(mid) <= mid: hi = mid
    else: lo = mid + 1
let rounds = lo

// Sort the row Ermek alone would leave, recording each swap as a pair of
// values. Swapping the same two values earlier has the same final effect.
let need = swaps_after(rounds)
var pairs: List<Int> = []
for v in 0..n:
    if val[v] != v:
        pairs.push(v)
        pairs.push(val[v])
        let home = slot_of[v]
        swap_slots(v, home)
reset()
print(rounds)
for r in 0..rounds:
    swap_slots(ermek[2 * r], ermek[2 * r + 1])
    if r < need:
        let (a, b) = (slot_of[pairs[2 * r]], slot_of[pairs[2 * r + 1]])
        print(a, b)
        swap_slots(a, b)
    else:
        print(0, 0)
