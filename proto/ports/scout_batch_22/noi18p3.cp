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
domain Letter = 'a'..='z'

struct State:
    len: Int   // its longest block
    link: Int  // the state of the longest tail that is elsewhere; the root links to itself
    next: [Letter]?Int
    first: Int  // where its blocks first end

/// The suffix automaton of the letters added so far. State 0 is the empty block.
struct Automaton:
    states: List<State>
    last: Int

    fn new() -> Automaton = Automaton([State(0, 0, [Letter] c => none, -1)], 0)

    /// Add the letter at place `at`; returns the state of the whole string so far.
    fn add(mut self, c: Letter, at: Int) -> Int:
        let cur = states.len
        states.push(State(states[last].len + 1, 0, [Letter] c => none, at))
        var p = last
        last = cur
        while true:
            if let q = states[p].next[c]:
                if states[q].len == states[p].len + 1: states[cur].link = q
                else:
                    // q also stands for longer blocks that do not end here: split it.
                    let clone = states.len
                    states.push(
                        State(
                            states[p].len + 1,
                            states[q].link,
                            states[q].next.copy(),
                            states[q].first,
                        )
                    )
                    states[q].link = clone
                    states[cur].link = clone
                    while states[p].next[c] == some(q):
                        states[p].next[c] = some(clone)
                        if p == 0: break
                        p = states[p].link
                break
            states[p].next[c] = some(cur)
            if p == 0: break
            p = states[p].link
        cur

let s: Str = read()
let n = s.len
var of_s = Automaton.new()

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
    let state = of_s.add(Letter(s[i]), i)
    while ends.len < of_s.states.len: ends.push(0)
    ends[state] = single(i, 0, n)
var longest_first = [v for v in 1..of_s.states.len]
longest_first.sort(by: v => -of_s.states[v].len)
for v in longest_first:
    let up = of_s.states[v].link
    ends[up] = union(ends[up], ends[v])

let q: Int = read()
for _ in 0..q:
    let (t, l, r): (Str, Int, Int) = read()
    let (from, to) = (l - 1, r)  // the block of s as places from..to
    var of_t = Automaton.new()
    var seen: List<Int> = []  // per place of t: the longest tail ending there that occurs
    var (at, len) = (0, 0)
    for i in 0..t.len:
        let c = Letter(t[i])
        of_t.add(c, i)
        while true:
            if let next = of_s.states[at].next[c]:
                // A block of len + 1 letters ending at place e starts at e - len.
                if any_in(ends[next], 0, n, from + len, to):
                    (at, len) = (next, len + 1)
                    break
            if len == 0: break
            len -= 1
            if len == of_s.states[of_s.states[at].link].len: at = of_s.states[at].link
        seen.push(len)
    var fresh = 0
    for v in 1..of_t.states.len:
        let state = of_t.states[v]
        fresh += max(0, state.len - max(of_t.states[state.link].len, seen[state.first]))
    print(fresh)
