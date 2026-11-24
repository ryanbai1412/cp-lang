// Source: NOI 2018 Your Name, https://dmoj.ca/problem/noi18p3
// limits: 4000 ms, 1024 MB
// Problem: a lowercase string s, |s| <= 5e5, and q <= 1e5 queries (t, l, r) with total |t| <= 1e6.
//   For each, print how many distinct substrings of t are not substrings of s[l..=r].
// Solution: a suffix automaton of s whose states keep their end positions in s as persistent
//   segment trees, merged up the suffix links without destroying the children. Feed t through it
//   keeping the longest suffix of t's prefix that occurs inside s[l..=r]: a step to state v with
//   current length len is allowed only if v has an end in l - 1 + len ..< r (0-based), else
//   shorten. Each state of t's own automaton covers lengths (link len, len] ending at one
//   position; the lengths above what matched there are new. O((|s| + sum |t|) log |s|).
// original: corpus/submissions/6513115.cpp
// Bounds: q: 1..=1e5; l, r: 1..=n
use std.segtree.{MergeSeg, MergeSegs}
use std.strings.{SuffixAutomaton}

domain Letter = 'a'..='z'

let s: Str = read()
let n = s.len  // <= 5e5
var of_s = SuffixAutomaton<Letter>.new()

domain Place = 0..n
var places = MergeSegs<Place>()
var ends: List<MergeSeg<Place>> = [MergeSeg<Place>()]  // per state of s: the places its blocks end
for i in 0..n:
    of_s.push(Letter(s[i]))
    while ends.len < of_s.len: ends.push(MergeSeg<Place>())
    ends[of_s.last] = places.singleton(Place(i))
let order = of_s.topo_order()
for i in (1..order.len).rev:
    let v = order[i]
    if let up = of_s.link(v):
        ends[up] = places.merge(ends[up], ends[v])

let q: 1..=1e5 = read()
repeat q:
    let (t, l, r): (Str, 1..=n, 1..=n) = read()
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
                if from + len < to and places.any_in(ends[next], Place(from + len)..=Place(to - 1)):
                    (at, len) = (next, len + 1)
                    break
            if len == 0: break
            len -= 1
            if let up = of_s.link(at) and len == of_s.longest(up): at = up
        seen.push(len)
    var fresh = 0
    for v in 1..of_t.len:
        let par_len = if let up = of_t.link(v): of_t.longest(up) else: 0
        fresh += max(0, of_t.longest(v) - max(par_len, seen[of_t.first_pos(v)!]))
    print(fresh)
