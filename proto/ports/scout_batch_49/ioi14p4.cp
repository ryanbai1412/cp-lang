// IOI 2014 Day 2 P1, Gondola
// original: corpus/submissions/4684567.cpp
const MOD = 1_000_000_009
type Z = Mod<MOD>

// Originals keep their cyclic order, so every surviving original fixes the
// same rotation: none when two disagree, else how far gondola 1 sits from slot 0.
// FRICTION: `n` and the array length are passed separately by the grader, so
// the domain is `0..n` by hand in all three signatures.
fn rotation(n: Int, seq: [0..n]Int) -> ?Int:
    let shifts = [(seq[i] - 1 - i) % n for i in 0..n if seq[i] <= n]
    if any(s != shifts[0] for s in shifts): return none
    shifts.first ?? (n - 1)

@grader
fn valid(n: Int, seq: [0..n]Int) -> Bool:
    rotation(n, seq) != none and all(a != b for (a, b) in sorted(seq).adjacent)

@grader
fn replacement(n: Int, seq: [0..n]Int) -> List<Int>:
    let shift = rotation(n, seq)!
    var now = [0..n] i => (i + shift) % n + 1
    // The slot with the largest number absorbs every replacement nobody else
    // ends on; a slot ending on g takes its final gondola at step g.
    let top = max_by(0..n, i => seq[i])!
    var ends_on = HashMap<Int, Int>()
    for (i, g) in seq.items: ends_on[g] = i
    var broken: List<Int> = []
    for g in n + 1..=seq[top]:
        let slot = ends_on.get(g) ?? top
        broken.push(now[slot])
        now[slot] = g
    broken

@grader
fn countReplacement(n: Int, seq: [0..n]Int) -> Z:
    if not valid(n, seq): return 0
    // Between consecutive final numbers a < b, each of the b - a - 1 hidden
    // replacements may land on any slot that still ends on something >= b.
    let added = sorted(g for g in seq if g > n)
    var ways: Z = if added.len == n: n else: 1
    for (i, (a, b)) in ([n] + added).adjacent.items:
        ways *= Z(added.len - i) ** (b - a - 1)
    ways
