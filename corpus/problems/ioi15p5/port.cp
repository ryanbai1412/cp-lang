// Source: IOI 2015 Day 2 P2 (Sorting), https://dmoj.ca/problem/ioi15p5
// Problem: a permutation S of 0..n-1 (n <= 2e5) and Ermek's m = 3n planned swaps (X[i], Y[i]).
//   Each round Ermek makes his next swap, then Aizhan swaps any two positions; she stops once S
//   is sorted. Print the least number of rounds R that sorts S, then Aizhan's R swaps.
// Solution: after Ermek's first t swaps the row needs n - cycles swaps, and swaps chosen as pairs
//   of values commute with his, so R is the least t with n - cycles(t) <= t: a binary search on
//   t, O(n + t) each. Sort the row Ermek alone leaves by value swaps, then replay the rounds: in
//   round r swap wherever those two values now sit. O((n + m) log m).
// original: corpus/submissions/5723358.cpp (grader function findSwapPairs)
// stdin: n, the permutation, m, then Ermek's m swaps. stdout: the number of
// rounds r, then our r swaps.
let n: 1..=200000 = read()
let start: [0..n]Int = read()
let m: 1..=600000 = read()
let ermek: [0..2 * m]Int = read()

var val: List<Int> = []      // val[slot]: scratch copy of the row
var slot_of: List<Int> = []  // inverse of val
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

let lo = (0..0).first_true(mid => swaps_after(mid) <= mid) ?? 0
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
