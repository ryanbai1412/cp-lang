// Source: ETSR 2023 Round 4 P1, https://dmoj.ca/problem/etsr23r4p1
// Problem: a permutation p of n positions; print a balanced bracket string in which p maps every
//   bracket to one of the opposite kind, or nothing if none exists.
// Solution: brackets alternate along each cycle of p, so an odd cycle has no answer and a
//   2-cycle opens at its smaller position. Each longer cycle has two phases; try every choice of
//   phases, in the order of the reference, and keep the first whose prefix sums stay
//   non-negative: O(2^m n) for m cycles longer than 2.
// original: corpus/submissions/5521420.cpp
// The statement is not public: n is the reference's int, and the phase search is fast only when
// few cycles are longer than 2.
let n: 1..=1e5 = read()
domain V = 1..=n
let p: [V]V = read()

fn brackets() -> ?Str:
    let cycles = p.perm_cycles()
    if cycles.any(c => c.len % 2 == 1): return None
    let (pairs, long) = cycles.partition(c => c.len == 2)
    var step: [V]Int = 0  // +1 opens, -1 closes
    for c in pairs:
        step[c[0]] = 1
        step[c[1]] = -1
    for phases in 0..(1 << long.len):
        for (j, c) in long.items:
            for (k, v) in c.items:
                step[v] = if (k % 2 == 1) != phases.bit(j): 1 else: -1
        let balance = step.scan(+)
        if V.cuts.all(c => balance[c] >= 0):
            return Some("".join([if s > 0: "(" else: ")" for s in step]))
    None

if let s = brackets(): print(s)
