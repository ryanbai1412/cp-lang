// Tuned: the cycles are walked by hand, a 2-cycle marked in place without a list of its own
// and only longer ones kept; each phase keeps a running balance instead of a prefix-sum list, and the answer is
// built from its characters, not by joining one string per bracket.
//
// Source: ETSR 2023 Round 4 P1, https://dmoj.ca/problem/etsr23r4p1
// Problem: a permutation p of n positions; print a balanced bracket string in which p maps every
//   bracket to one of the opposite kind, or nothing if none exists.
// Solution: brackets alternate along each cycle of p, so an odd cycle has no answer and a
//   2-cycle opens at its smaller position. Each longer cycle has two phases; try every choice of
//   phases, in the order of the reference, and keep the first whose prefix sums stay
//   non-negative: O(2^m n) for m cycles longer than 2.
// Bounds: n: 1..=1e5; p: 1..=n
// original: corpus/submissions/5521420.cpp
// The statement is not public: n is the reference's int, and the phase search is fast only when
// few cycles are longer than 2.
let n: 1..=1e5 = read()
domain V = 1..=n
let p: [V]V = read()

fn brackets() -> ?Str:
    var step: [V]Int = 0  // +1 opens, -1 closes
    var seen: [V]Bool = false
    var long = List<List<V>>()
    for s in V:
        if seen[s]: continue
        let t = p[s]
        if t != s and p[t] == s:
            seen[s] = true
            seen[t] = true
            step[min(s, t)] = 1
            step[max(s, t)] = -1
            continue
        var c = List<V>()
        var v = s
        while not seen[v]:
            seen[v] = true
            c.push(v)
            v = p[v]
        if c.len % 2 == 1: return None
        long.push(c)
    for phases in 0..(1 << long.len):
        for (j, c) in long.items:
            for (k, v) in c.items:
                step[v] = if (k % 2 == 1) != phases.bit(j): 1 else: -1
        var balance = 0
        var low = 0
        for v in V:
            balance += step[v]
            low min= balance
        if low >= 0: return Some(Str([if s > 0: '(' else: ')' for s in step]))
    None

if let s = brackets(): print(s)
