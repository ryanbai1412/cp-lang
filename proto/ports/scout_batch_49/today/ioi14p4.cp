// IOI 2014 Day 2 P1, Gondola (buildable today-form)
// original: corpus/submissions/4684567.cpp (grader functions valid / replacement / countReplacement)
// stdin: task (1 valid, 2 replacement, 3 count), n, then the n gondola numbers.
const MOD = 1000000009
let task: 1..=3 = read()
let n: 1..=100000 = read()
let seq: [0..n]Int = read()

// Originals keep their cyclic order, so every surviving original fixes the
// same rotation: the slot that held gondola 1 (as a slot offset).
var shift = -1
var ok = true
for i in 0..n:
    if seq[i] <= n:
        let s = (seq[i] - 1 - i) % n
        if shift != -1 and shift != s: ok = false
        shift = s
var sorted_seq: List<Int> = [seq[i] for i in 0..n]
sort(mut sorted_seq)
for i in 1..n:
    if sorted_seq[i] == sorted_seq[i - 1]: ok = false

fn power(base: Int, exp: Int) -> Int:
    var r = 1
    var b = base % MOD
    var e = exp
    while e > 0:
        if e % 2 == 1: r = r * b % MOD
        b = b * b % MOD
        e = e / 2
    r

if task == 1:
    print(Int(ok))
elif task == 2:
    // The slot with the largest number absorbs every replacement nobody else
    // ends on; a slot ending on g takes its final gondola at step g.
    if shift == -1: shift = n - 1
    var now: List<Int> = [(i + shift) % n + 1 for i in 0..n]
    var top = 0
    for i in 0..n:
        if seq[i] > seq[top]: top = i
    var ends_on = HashMap<Int, Int>()
    for i in 0..n: ends_on[seq[i]] = i
    var broken: List<Int> = []
    for g in n + 1..=seq[top]:
        let slot = ends_on.get(g) ?? top
        broken.push(now[slot])
        now[slot] = g
    print(broken.len)
    print(broken)
else:
    // Between consecutive final numbers a < b, each of the b - a - 1 hidden
    // replacements may land on any slot that still ends on something >= b.
    var ways = 0
    if ok:
        ways = if shift == -1: n else: 1
        var prev = n
        var open = count(seq[i] > n for i in 0..n)
        for g in sorted_seq:
            if g > n:
                ways = ways * power(open, g - prev - 1) % MOD
                prev = g
                open -= 1
    print(ways)
