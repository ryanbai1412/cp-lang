// Source: IOI 2016 P5 (Unscrambling a Messy Bug), https://dmoj.ca/problem/ioi16p5
// Problem: interactive. A hidden permutation p of n bits (n = 2^b <= 128). Add at most w
//   n-bit strings to a set, compile once (every string's bits get reordered by p), then ask
//   at most r membership queries; recover p. Hardest subtask: n = 128, w = r = 896.
// Solution: split the positions by one index bit per level. At level d, inside each group,
//   write the string with 1s outside the group and 0s inside except at a, for each a whose
//   bit d is set. After compiling, ask the same string for every position a of each image
//   group: it is in the set exactly when a's preimage has bit d set, which sets that bit of
//   the answer and splits the group the same way. (n/2) log n writes, n log n reads.
// Stdio-adapted: "A s", "C" and "Q s" are the three library calls, "!" prints the answer.
// Bounds: n: 2..=128; w, r: 1..=10000
// original: corpus/submissions/4692325.cpp
let (n, w, r): (2..=128, 1..=10000, 1..=10000) = read()
domain Pos = 0..n
domain Level = 0..(0..8).count(d => 1 << d < n)
assert n / 2 * Level.count <= w and n * Level.count <= r

// 1 outside the group, 0 inside except at `a`
fn shape(group: List<Pos>, a: Pos) -> [Pos]Char:
    var s: [Pos]Char = '1'
    for p in group if p != a: s[p] = '0'
    s

fn write(s: [Pos]Char):
    let ack: Int = ask("A", s)
    assert ack == 1

fn member(s: [Pos]Char) -> Bool:
    let hit: 0..=1 = ask("Q", s)
    hit == 1

var groups = [Pos.collect()]
for d in Level:
    var next: List<List<Pos>> = []
    for g in groups:
        let (high, low) = g.partition(a => a.ord.bit(d.ord))
        for a in high: write(shape(g, a))
        next += [high, low]
    groups = next
let compiled: Int = ask("C")
assert compiled == 1

var answer: [Pos]Int = 0
groups = [Pos.collect()]
for d in Level:
    var next: List<List<Pos>> = []
    for g in groups:
        let (high, low) = g.partition(a => member(shape(g, a)))
        for a in high: answer[a] += 1 << d.ord
        next += [high, low]
    groups = next
print("!", answer)
