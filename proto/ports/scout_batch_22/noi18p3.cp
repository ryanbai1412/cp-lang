// NOI 2018 Your Name: a string s (up to 500000 lowercase letters) and q
// questions, each a string t and a block l..=r of s, 1-based (the t's add up
// to at most 1000000 letters). For each, print how many different blocks of
// t do not occur anywhere inside s[l..=r].
//
// A suffix automaton of s, where every state also knows the set of places
// its blocks end in s: a mergeable segment tree per state, merged up the
// suffix links without destroying the children's trees. Feed t through it,
// keeping the longest tail of t so far that occurs inside s[l..=r]; a step
// is allowed only if the next state has an end place far enough right of l
// and at most r. Then every state of t's own automaton stands for the tails
// of one place of t with lengths in a range, and the ones longer than what
// occurred at that place are new.
// original: corpus/submissions/6513115.cpp
use std.strings.{SuffixAutomaton}

domain Letter = 'a'..='z'

let s: Str = read()
let n = s.len
var of_s = SuffixAutomaton<Letter>.new()

// Sets of places 0..n as segment trees that share nodes. Node 0 is the empty set.
var left = [0]
var right = [0]
fn single(at: Int, lo: Int, hi: Int) -> Int:
    left.push(0)
    right.push(0)
    let node = left.len - 1
    if hi - lo > 1:
        let mid = (lo + hi) / 2
        if at < mid: left[node] = single(at, lo, mid)
        else: right[node] = single(at, mid, hi)
    node

fn union(a: Int, b: Int) -> Int:
    if a == 0 or b == 0: return a + b
    let (l, r) = (union(left[a], left[b]), union(right[a], right[b]))
    left.push(l)
    right.push(r)
    left.len - 1

/// Does the set hold a place in from..to?
fn any_in(node: Int, lo: Int, hi: Int, from: Int, to: Int) -> Bool:
    if node == 0 or from >= hi or to <= lo or from >= to: return false
    if from <= lo and hi <= to: return true
    let mid = (lo + hi) / 2
    any_in(left[node], lo, mid, from, to) or any_in(right[node], mid, hi, from, to)

var ends: List<Int> = [0]  // per state of s: the places its blocks end
for i in 0..n:
    of_s.push(Letter(s[i]))
    while ends.len < of_s.len: ends.push(0)
    ends[of_s.last] = single(i, 0, n)
let order = of_s.topo_order()
for i in (1..order.len).rev:
    let v = order[i]
    if let up = of_s.link(v):
        ends[up] = union(ends[up], ends[v])

let q: Int = read()
for _ in 0..q:
    let (t, l, r): (Str, Int, Int) = read()
    let (from, to) = (l - 1, r)  // the block of s as places from..to
    var of_t = SuffixAutomaton<Letter>.new()
    var seen: List<Int> = []  // per place of t: the longest tail ending there that occurs
    var (at, len) = (of_s.root, 0)
    for ch in t:
        let c = Letter(ch)
        of_t.push(c)
        while true:
            if let next = of_s.step(at, c):
                // A block of len + 1 letters ending at place e starts at e - len.
                if any_in(ends[next], 0, n, from + len, to):
                    (at, len) = (next, len + 1)
                    break
            if len == 0: break
            len -= 1
            if let up = of_s.link(at) and len == of_s.longest(up): at = up
        seen.push(len)
    var fresh = 0
    for v in 1..of_t.len:
        let par_len = if let up = of_t.link(v): of_t.longest(up) else: 0
        fresh += max(0, of_t.longest(v) - max(par_len, seen[of_t.first_pos(v)]))
    print(fresh)
