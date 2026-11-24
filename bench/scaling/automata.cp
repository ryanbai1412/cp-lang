// n: 512
// fixed: A
use std.strings.{KmpAutomaton, SuffixAutomaton}
domain L = 0..4
let n: Int = read()
let ws = [[L((i * 7919 + i / 3) % 3) for i in 0..n] for _ in 0..2]
let pats = [[L((i + j) % 2) for i in 0..n] for j in 0..2]
let probe = [L(i % 2) for i in 0..8]
var acc = 0
for r in 0..1024:
    let w = ws[r % 2]
    acc += KmpAutomaton<L>(pats[r % 2]).len // timed: strings.KmpAutomaton.new
    let m = SuffixAutomaton<L>.of(w) // timed: strings.SuffixAutomaton.of
    for _ in 0..8:
        acc += m.distinct() // timed: strings.SuffixAutomaton.distinct
    acc += m.topo_order().len // timed: strings.SuffixAutomaton.topo_order
    acc += m.occ_counts().len // timed: strings.SuffixAutomaton.occ_counts
    for v in m.states(): acc += v // timed: strings.SuffixAutomaton.states
    acc += m.endpos_tree().len // timed: strings.SuffixAutomaton.endpos_tree
    if let x = m.kth_substring(r * 7): acc += x.len // timed: strings.SuffixAutomaton.kth_substring
    for _ in 0..4:
        acc += m.lcs(ws[1 - r % 2]).len // timed: strings.SuffixAutomaton.lcs
        if let v = m.find(w): acc += v // timed: strings.SuffixAutomaton.find
        if m.contains(w): acc += 1 // timed: strings.SuffixAutomaton.contains
    var g = SuffixAutomaton<L>()
    for c in w:
        g.push(c) // timed: strings.SuffixAutomaton.push
    g.reset()
    g.add_word(w) // timed: strings.SuffixAutomaton.add_word
    acc += g.len
let kmp = KmpAutomaton<L>(probe)
let m0 = SuffixAutomaton<L>.of(ws[0])
let states = m0.len
var g0 = SuffixAutomaton<L>.of(probe)
for r in 0..2048:
    for k in kmp.find_all(ws[r % 2]): acc += k // timed: strings.KmpAutomaton.find_all
for i in 0..1 << 18:
    let v = i * 7 % states
    let c = L(i % 3)
    acc += kmp.step(i % 9, c) // timed: strings.KmpAutomaton.step
    acc += kmp.link(i % 9) // timed: strings.KmpAutomaton.link
    acc += kmp.len // timed: strings.KmpAutomaton.len
    acc += SuffixAutomaton<L>().len // timed: strings.SuffixAutomaton.new
    g0.reset() // timed: strings.SuffixAutomaton.reset
    acc += m0.root // timed: strings.SuffixAutomaton.root
    acc += m0.len // timed: strings.SuffixAutomaton.len
    if let t = m0.step(v, c): acc += t // timed: strings.SuffixAutomaton.step
    if let t = m0.link(v): acc += t // timed: strings.SuffixAutomaton.link
    acc += m0.longest(v) // timed: strings.SuffixAutomaton.longest
    acc += m0.shortest(v) // timed: strings.SuffixAutomaton.shortest
    if let t = m0.first_pos(v): acc += t // timed: strings.SuffixAutomaton.first_pos
print(acc)
