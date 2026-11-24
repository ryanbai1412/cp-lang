// Source: IOI 2014 Day 2 P1 (Gondola), https://dmoj.ca/problem/ioi14p4
// Problem: n <= 1e5 gondolas on a circle, first numbered 1..n in cyclic order; each failure
//   replaces a gondola by a new one numbered n+1, n+2, ... Given the final numbers (<= 250000)
//   read from some position: task 1, is it a valid gondola sequence; task 2, print a
//   replacement sequence (the broken numbers in order); task 3, count the replacement
//   sequences giving it, mod 1e9 + 9 (0 if invalid).
// Solution: valid iff the numbers are distinct and every surviving original (<= n) implies the
//   same rotation. Task 2: fix that rotation (any if none survive); the slot ending on the
//   largest number takes every number nobody ends on, so walk g = n+1..max and replace slot
//   g's (or that one's) current gondola. Task 3: sort the final numbers > n; each hidden number
//   between consecutive finals may go to any slot still ending on something larger, so
//   multiply open^(gap), times n if no original survives. O(n log n).
// original: corpus/submissions/4684567.cpp (grader functions valid, replacement, countReplacement)
// stdin: task (1 valid, 2 replacement, 3 count), n, then the n gondola numbers.
// Bounds: task: 1..=3; n: 1..=100000; seq: 1..=MOST
type Z = Mod<1_000_000_009>
const MOST = 250_000
let task: 1..=3 = read()
let n: 1..=1e5 = read()
domain Slot = 1..=n
let seq: [Slot]1..=MOST = read()

// Originals keep their cyclic order, so every surviving original must fix the
// same rotation.
let shifts = [(seq[i] - i.label) % n for i in Slot if seq[i] <= n]
let shift = shifts.first
let ok = seq.sorted().deduped().len == n and shifts.all(s => s == shift!)

// The slot with the largest number absorbs every replacement nobody else
// ends on; a slot ending on g takes its final gondola at step g.
fn replacement() -> List<Int>:
    let s = shift ?? (n - 1)
    var now = [Slot] i => (i.label - 1 + s) % n + 1
    var slot_of: [1..=MOST]?Slot = None
    for i in Slot: slot_of[seq[i]] = Some(i)
    let last = seq.max()!
    let top = slot_of[last]!
    var broken: List<Int> = []
    for g in n + 1..=last:
        let slot = slot_of[g] ?? top
        broken.push(now[slot])
        now[slot] = g
    broken

// Between consecutive final numbers a < b, each of the b - a - 1 hidden
// replacements may land on any slot that still ends on something >= b.
fn ways() -> Z:
    if not ok: return 0
    var ways: Z = if shift == None: Z(n) else: 1
    var prev = n
    var open = count(x > n for x in seq)
    for g in seq.sorted() if g > n:
        ways *= Z(open) ** (g - prev - 1)
        prev = g
        open -= 1
    ways

if task == 1: print(Int(ok))
elif task == 2:
    let broken = replacement()
    print(broken.len)
    print(broken)
else: print(ways())
