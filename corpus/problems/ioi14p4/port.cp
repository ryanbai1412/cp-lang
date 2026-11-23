// IOI 2014 Day 2 P1, Gondola
// original: corpus/submissions/4684567.cpp (grader functions valid / replacement / countReplacement)
// stdin: task (1 valid, 2 replacement, 3 count), n, then the n gondola numbers.
type Z = Mod<1_000_000_009>
let task: 1..=3 = read()
let n: 1..=100000 = read()
let seq: [1..=n]Int = read()

// Originals keep their cyclic order, so every surviving original fixes the
// same rotation.
var shift: ?Int = None
var ok = true
for i in 1..=n if seq[i] <= n:
    let s = (seq[i] - i) % n
    if let t = shift, t != s: ok = false
    shift = s
let sorted_seq = seq.sorted()
if sorted_seq.deduped().len < n: ok = false

if task == 1:
    print(Int(ok))
    return

if task == 2:
    // The slot with the largest number absorbs every replacement nobody else
    // ends on; a slot ending on g takes its final gondola at step g.
    let s = shift ?? (n - 1)
    var now = [1..=n] i => (i - 1 + s) % n + 1
    var slot_of = HashMap<Int, Int>()
    for i in 1..=n: slot_of[seq[i]] = i
    let last = seq.max()!
    let top = slot_of[last]
    var broken: List<Int> = []
    for g in n + 1..=last:
        let slot = slot_of.get(g) ?? top
        broken.push(now[slot])
        now[slot] = g
    print(broken.len)
    print(broken)
    return

// Between consecutive final numbers a < b, each of the b - a - 1 hidden
// replacements may land on any slot that still ends on something >= b.
if not ok:
    print(0)
    return
var ways: Z = if shift == None: Z(n) else: 1
var prev = n
var open = count(x > n for x in seq)
for g in sorted_seq if g > n:
    ways *= Z(open) ** (g - prev - 1)
    prev = g
    open -= 1
print(ways)
